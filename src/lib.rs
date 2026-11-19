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

extern crate alloc;

use alloc::{vec, vec::Vec};
use sas_plus::{Abstraction, State};

/// The result of partitioning costs among abstractions in a fixed order.
pub struct CostPartitionedHeuristic<'a, A> {
    abstractions: &'a [A],
    /// Goal distances of each abstraction under its share of the costs.
    distances: Vec<Vec<Option<u64>>>,
    /// The cost function assigned to each abstraction.
    cost_functions: Vec<Vec<u64>>,
}

impl<A: Abstraction> CostPartitionedHeuristic<'_, A> {
    /// Sum of the abstraction heuristics; `None` if any detects a dead end.
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

    pub fn cost_functions(&self) -> &[Vec<u64>] {
        &self.cost_functions
    }
}

/// Saturated cost partitioning: each abstraction in `order` receives only the
/// costs it needs to keep its goal distances (its *saturated* cost
/// function); the rest goes to later abstractions. Abstractions not in
/// `order` get zero costs.
pub fn saturated_cost_partitioning<'a, A: Abstraction>(
    abstractions: &'a [A],
    order: &[usize],
    costs: &[u64],
) -> CostPartitionedHeuristic<'a, A> {
    let mut remaining = costs.to_vec();
    let mut distances: Vec<Vec<Option<u64>>> = vec![Vec::new(); abstractions.len()];
    let mut cost_functions = vec![vec![0; costs.len()]; abstractions.len()];

    for &i in order {
        let lts = abstractions[i].lts();
        let h = lts.goal_distances(&remaining);
        let mut saturated = vec![0u64; costs.len()];
        for t in &lts.transitions {
            if let (Some(hs), Some(hd)) = (h[t.src], h[t.dst]) {
                saturated[t.label] = saturated[t.label].max(hs.saturating_sub(hd));
            }
        }
        for (r, s) in remaining.iter_mut().zip(&saturated) {
            debug_assert!(*s <= *r, "saturated cost exceeds remaining cost");
            *r -= s;
        }
        distances[i] = h;
        cost_functions[i] = saturated;
    }
    // Abstractions not in `order` still detect dead ends under zero costs.
    let zero = vec![0; costs.len()];
    for (a, d) in abstractions.iter().zip(&mut distances) {
        if d.is_empty() {
            *d = a.lts().goal_distances(&zero);
        }
    }
    CostPartitionedHeuristic {
        abstractions,
        distances,
        cost_functions,
    }
}

/// Maximum over several cost-partitioned heuristics (one per order).
pub struct MaxOverOrders<'a, A> {
    abstractions: &'a [A],
    heuristics: Vec<CostPartitionedHeuristic<'a, A>>,
}

impl<'a, A: Abstraction> MaxOverOrders<'a, A> {
    pub fn new(abstractions: &'a [A], orders: &[Vec<usize>], costs: &[u64]) -> Self {
        Self {
            abstractions,
            heuristics: orders
                .iter()
                .map(|o| saturated_cost_partitioning(abstractions, o, costs))
                .collect(),
        }
    }

    /// The cost-partitioned heuristic of each order.
    pub fn heuristics(&self) -> &[CostPartitionedHeuristic<'a, A>] {
        &self.heuristics
    }

    /// Every component is admissible, so any component's dead end is a dead end.
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
