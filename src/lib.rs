//! Cost partitioning over arbitrary [`Abstraction`]s (projections,
//! merge-and-shrink, Cartesian abstractions, ...).
//!
//! Splitting each operator's cost among several abstractions, so that the
//! parts sum to at most the original cost, makes the *sum* of their
//! heuristics admissible.
//!
//! Status: saturated cost partitioning (Seipp, Keller & Helmert 2020) for a
//! given order, and maximizing over several orders. Planned: greedy and
//! diversified orders, general (negative) costs, an optimal LP baseline.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{vec, vec::Vec};
use sas_plus::{Abstraction, LabeledTransitionSystem, State};

/// The result of partitioning costs among abstractions in a fixed order.
///
/// Built by [`saturated_cost_partitioning`]. Its [`value`](Self::value) is the
/// sum of the abstractions' goal distances under their shares of the costs,
/// which is admissible because the shares sum to at most the original costs.
pub struct CostPartitionedHeuristic<'a, A> {
    /// The abstractions the costs were partitioned among.
    abstractions: &'a [A],
    /// Goal distances of each abstraction under its share of the costs.
    ///
    /// Indexed like `abstractions`, then by abstract state; `None` marks an
    /// abstract state from which no goal is reachable.
    distances: Vec<Vec<Option<u64>>>,
    /// The cost function assigned to each abstraction, indexed like
    /// `abstractions`, then by operator id.
    cost_functions: Vec<Vec<u64>>,
}

impl<A: Abstraction> CostPartitionedHeuristic<'_, A> {
    /// Sum of the abstraction heuristics; `None` if any detects a dead end.
    ///
    /// A state that some abstraction prunes ([`Abstraction::abstract_state`]
    /// returns `None`) is a dead end too.
    #[must_use]
    pub fn value(&self, state: &State) -> Option<u64> {
        self.abstractions
            .iter()
            .zip(&self.distances)
            .try_fold(0, |sum, (a, d)| Some(sum + d[a.abstract_state(state)?]?))
    }

    /// Like [`value`](Self::value), given the abstract state of `state` in
    /// each abstraction.
    fn value_of(&self, abstract_states: &[usize]) -> Option<u64> {
        debug_assert_eq!(abstract_states.len(), self.distances.len());

        abstract_states
            .iter()
            .zip(&self.distances)
            .try_fold(0, |sum, (&s, d)| Some(sum + d[s]?))
    }

    /// The cost function assigned to each abstraction.
    ///
    /// Indexed like the abstractions, then by operator id. For every operator,
    /// the entries sum to at most its original cost. Abstractions that were
    /// not in the order have all-zero cost functions.
    #[must_use]
    pub fn cost_functions(&self) -> &[Vec<u64>] {
        &self.cost_functions
    }
}

/// Saturated cost partitioning: each abstraction in `order` receives only the
/// costs it needs to keep its goal distances (its *saturated* cost
/// function); the rest goes to later abstractions. Abstractions not in
/// `order` get zero costs.
///
/// `costs` holds the cost of each operator, indexed by operator id, which is
/// the label of the abstractions' transitions.
///
/// # Panics
///
/// Panics if `order` contains an index outside `abstractions` or the same
/// index twice, or if `costs.len()` differs from an abstraction's number of
/// labels.
pub fn saturated_cost_partitioning<'a, A: Abstraction>(
    abstractions: &'a [A],
    order: &[usize],
    costs: &[u64],
) -> CostPartitionedHeuristic<'a, A> {
    let mut remaining = costs.to_vec();
    let mut distances: Vec<Vec<Option<u64>>> = vec![Vec::new(); abstractions.len()];
    let mut cost_functions = vec![vec![0; costs.len()]; abstractions.len()];
    let mut seen = vec![false; abstractions.len()];

    for &i in order {
        // A second pass would overwrite the first one's distances while its costs stay spent.
        assert!(
            !core::mem::replace(&mut seen[i], true),
            "saturated_cost_partitioning: abstraction {i} appears twice in order"
        );
        let lts = abstractions[i].lts();
        let h = lts.goal_distances(&remaining);
        let saturated = saturated_costs(lts, &h);
        for (r, s) in remaining.iter_mut().zip(&saturated) {
            debug_assert!(*s <= *r, "saturated cost exceeds remaining cost");
            *r -= s;
        }

        distances[i] = h;
        cost_functions[i] = saturated;
    }

    fill_skipped_distances(abstractions, &mut distances, costs.len());

    CostPartitionedHeuristic {
        abstractions,
        distances,
        cost_functions,
    }
}

/// The saturated cost function of `lts` for its goal distances `h`.
///
/// Each label gets the largest drop in goal distance along one of its
/// transitions: the least cost that keeps every goal distance in `h`.
/// Transitions from or to dead states (`None`) need no cost.
fn saturated_costs(lts: &LabeledTransitionSystem, h: &[Option<u64>]) -> Vec<u64> {
    let mut saturated = vec![0u64; lts.num_labels];
    for t in &lts.transitions {
        if let (Some(hs), Some(hd)) = (h[t.src], h[t.dst]) {
            saturated[t.label] = saturated[t.label].max(hs.saturating_sub(hd));
        }
    }

    saturated
}

/// Fills in zero-cost goal distances for the abstractions the order skipped.
///
/// Their distances are empty until then. Under zero costs, they are `0` or
/// `None`, so these abstractions add nothing but still detect dead ends.
fn fill_skipped_distances<A: Abstraction>(
    abstractions: &[A],
    distances: &mut [Vec<Option<u64>>],
    num_labels: usize,
) {
    let zero = vec![0; num_labels];
    for (a, d) in abstractions.iter().zip(distances) {
        if d.is_empty() {
            *d = a.lts().goal_distances(&zero);
        }
    }
}

/// Maximum over several cost-partitioned heuristics (one per order).
///
/// Each order yields an admissible heuristic, so their maximum is admissible
/// too, and at least as informed as each of them.
pub struct MaxOverOrders<'a, A> {
    /// The abstractions every order partitions costs among.
    abstractions: &'a [A],
    /// One saturated cost partitioning per order, all over `abstractions`.
    heuristics: Vec<CostPartitionedHeuristic<'a, A>>,
}

impl<'a, A: Abstraction> MaxOverOrders<'a, A> {
    /// Runs [`saturated_cost_partitioning`] once for each of `orders`.
    ///
    /// # Panics
    ///
    /// Panics if `orders` is empty, and under the same conditions as
    /// [`saturated_cost_partitioning`], for any of the orders.
    #[must_use]
    pub fn new(abstractions: &'a [A], orders: &[Vec<usize>], costs: &[u64]) -> Self {
        // With no orders, `value` would be 0 everywhere and miss dead ends.
        assert!(
            !orders.is_empty(),
            "MaxOverOrders::new needs at least one order"
        );
        Self {
            abstractions,
            heuristics: orders
                .iter()
                .map(|o| saturated_cost_partitioning(abstractions, o, costs))
                .collect(),
        }
    }

    /// The cost-partitioned heuristic of each order.
    #[must_use]
    pub fn heuristics(&self) -> &[CostPartitionedHeuristic<'a, A>] {
        &self.heuristics
    }

    /// The largest value over all orders; `None` if `state` is a dead end.
    ///
    /// Every component is admissible, so any component's dead end is a dead end.
    #[must_use]
    pub fn value(&self, state: &State) -> Option<u64> {
        // Abstract states do not depend on the order: compute them once.
        let abstract_states: Vec<usize> = self
            .abstractions
            .iter()
            .map(|a| a.abstract_state(state))
            .collect::<Option<_>>()?;

        self.heuristics
            .iter()
            .try_fold(0, |best, h| Some(best.max(h.value_of(&abstract_states)?)))
    }
}

/// Doctests for the code examples in README.md (not part of the API).
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_doctests {}
