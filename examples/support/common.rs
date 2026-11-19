//! Helpers shared by the examples: reading a task and building projections and orders.

use sas_plus::{Projection, Task};

/// Reads the translator output at `path`, naming the file when that fails.
pub fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| {
        format!("cannot read {path}: {e}; pass a translator output.sas as the argument")
    })
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

/// Up to `count` distinct orders over `len` abstractions: rotations of the identity.
///
/// There are only `len` rotations, so fewer orders come back when `len < count`.
pub fn rotations(len: usize, count: usize) -> Vec<Vec<usize>> {
    (0..count.min(len))
        .map(|shift| (0..len).map(|i| (i + shift) % len).collect())
        .collect()
}
