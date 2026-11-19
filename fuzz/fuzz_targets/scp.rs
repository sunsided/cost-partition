//! Fuzzes saturated cost partitioning on random tasks against the exact `h*`.
//!
//! The fuzzer picks the task (seed and features), the order of the
//! projections and how much of it to use. Every heuristic must stay
//! admissible, never claim a false dead end, and partition the costs.

#![no_main]

use arbitrary::Arbitrary;
use cost_partition::saturated_cost_partitioning;
use libfuzzer_sys::fuzz_target;
use sas_plus::random::{RandomTaskParams, random_task};
use sas_plus::{Projection, Task};

/// One fuzz case: which task to generate and in which order to partition.
#[derive(Arbitrary, Debug)]
struct Input {
    /// Seed for [`random_task`].
    seed: u64,
    /// Whether the task has conditional effects and axioms.
    rich: bool,
    /// Sort keys for the projections; missing keys count as 0, ties keep index order.
    keys: Vec<u8>,
    /// Order prefix length, taken modulo the number of projections plus one.
    prefix: u8,
}

/// Generator parameters: plain SAS+, or with conditional effects and axioms.
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
fn atomic_and_pair_projections(task: &Task) -> Vec<Projection> {
    let n = task.num_variables();
    let singles = (0..n).map(|v| vec![v]);
    let pairs = (0..n).flat_map(|a| (a + 1..n).map(move |b| vec![a, b]));

    singles
        .chain(pairs)
        .map(|pattern| Projection::new(task, pattern))
        .collect()
}

/// The order `0..len` sorted by `keys`, then cut to `prefix % (len + 1)` entries.
fn order(keys: &[u8], len: usize, prefix: u8) -> Vec<usize> {
    let key = |i: usize| keys.get(i).copied().unwrap_or(0);
    let mut order: Vec<usize> = (0..len).collect();
    order.sort_by_key(|&i| key(i));
    order.truncate(usize::from(prefix) % (len + 1));

    order
}

fuzz_target!(|input: Input| {
    let task = random_task(input.seed, params(input.rich));
    let costs = task.costs();
    let projections = atomic_and_pair_projections(&task);
    let order = order(&input.keys, projections.len(), input.prefix);
    let init = task.initial_state();

    let scp = saturated_cost_partitioning(&projections, &order, &costs);
    for (label, &cost) in costs.iter().enumerate() {
        let shared: u64 = scp.cost_functions().iter().map(|cf| cf[label]).sum();
        assert!(shared <= cost, "operator {label}: {shared} > {cost}");
    }

    match (scp.value(&init), task.perfect_heuristic()(&init)) {
        (Some(h), Some(opt)) => assert!(h <= opt, "SCP {h} > h* {opt}"),
        (None, h_star) => assert_eq!(h_star, None, "SCP claims a false dead end"),
        (Some(_), None) => {}
    }
});
