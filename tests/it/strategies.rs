//! proptest strategies: random planning tasks with projections and an order.

use proptest::prelude::*;
use sas_plus::random::{RandomTaskParams, random_task};
use sas_plus::{Projection, Task};

/// A random task, its projections, and an order over them to partition costs in.
#[derive(Debug)]
pub struct Case {
    /// The planning task; generated from a seed by [`random_task`].
    pub task: Task,
    /// Projections onto every single variable and every pair of variables.
    pub projections: Vec<Projection>,
    /// A permutation of `0..projections.len()`.
    pub order: Vec<usize>,
    /// Length of an order prefix, in `0..=order.len()`, for partial orders.
    pub prefix: usize,
}

/// Generator parameters: plain SAS+, or with conditional effects and axioms.
///
/// Projections over-approximate both features, so both variants must stay
/// admissible.
fn params(rich: bool) -> RandomTaskParams {
    let plain = RandomTaskParams::default();
    if rich {
        RandomTaskParams {
            max_conditions: 2,
            num_derived: 2,
            ..plain
        }
    } else {
        plain
    }
}

/// Projections onto every variable and every pair of variables of `task`.
pub fn atomic_and_pair_projections(task: &Task) -> Vec<Projection> {
    let n = task.num_variables();
    let singles = (0..n).map(|v| vec![v]);
    let pairs = (0..n).flat_map(|a| (a + 1..n).map(move |b| vec![a, b]));

    singles
        .chain(pairs)
        .map(|pattern| Projection::new(task, pattern))
        .collect()
}

/// Random cases from any seed, with and without conditional effects and axioms.
pub fn case() -> impl Strategy<Value = Case> {
    (any::<u64>(), any::<bool>()).prop_flat_map(|(seed, rich)| {
        let task = random_task(seed, params(rich));
        let projections = atomic_and_pair_projections(&task);
        let identity: Vec<usize> = (0..projections.len()).collect();
        let len = identity.len();

        (
            Just(task),
            Just(projections),
            Just(identity).prop_shuffle(),
            0..=len,
        )
            .prop_map(|(task, projections, order, prefix)| Case {
                task,
                projections,
                order,
                prefix,
            })
    })
}
