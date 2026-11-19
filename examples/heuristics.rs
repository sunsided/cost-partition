//! Why cost partitioning: compares ways of combining projection heuristics.
//!
//! Each projection alone is admissible but weak. Adding their estimates is
//! stronger but can overestimate, because every projection charges the full
//! cost of the same operators. Saturated cost partitioning (SCP) splits each
//! operator's cost among the projections first, so the sum stays admissible.
//!
//! ```sh
//! cargo run --example heuristics                    # translated gripper-4
//! cargo run --example heuristics -- path/to/output.sas
//! ```

use std::error::Error;

use cost_partition::{MaxOverOrders, saturated_cost_partitioning};
use sas_plus::fdr::SasFile;
use sas_plus::{Abstraction, Projection, State, Task};

/// Translator output used when no path is given.
const DEFAULT_TASK: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/translated/gripper-4/output.sas"
);

/// Number of orders the maximum over orders uses.
const ORDERS: usize = 8;

/// Prints the initial-state estimate of each combination next to the optimum.
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_TASK.into());
    let task = SasFile::parse(&std::fs::read_to_string(&path)?)?.to_task();
    let costs = task.costs();
    let init = task.initial_state();
    let projections = atomic_and_pair_projections(&task);

    let single = projections
        .iter()
        .map(|p| full_cost_estimate(p, &init, &costs))
        .max()
        .flatten();
    let naive_sum: Option<u64> = projections
        .iter()
        .map(|p| full_cost_estimate(p, &init, &costs))
        .sum();
    let order: Vec<usize> = (0..projections.len()).collect();
    let scp = saturated_cost_partitioning(&projections, &order, &costs).value(&init);
    let max = MaxOverOrders::new(&projections, &rotations(projections.len(), ORDERS), &costs)
        .value(&init);

    let h_star = task.perfect_heuristic()(&init);

    println!("{path}: {} projections\n", projections.len());
    report("optimal plan cost (h*)", h_star, h_star);
    report("best single projection", single, h_star);
    report("sum, full costs", naive_sum, h_star);
    report("SCP, one order", scp, h_star);
    report(&format!("max of SCP over {ORDERS} orders"), max, h_star);

    Ok(())
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

/// Goal distance of `state` in `abstraction` under the original `costs`.
///
/// `None` means the abstraction proves `state` a dead end.
fn full_cost_estimate(abstraction: &Projection, state: &State, costs: &[u64]) -> Option<u64> {
    let abstract_state = abstraction.abstract_state(state)?;

    abstraction.lts().goal_distances(costs)[abstract_state]
}

/// `count` distinct orders over `len` abstractions: rotations of the identity.
fn rotations(len: usize, count: usize) -> Vec<Vec<usize>> {
    (0..count.min(len))
        .map(|shift| (0..len).map(|i| (i + shift) % len).collect())
        .collect()
}

/// Prints one row of the comparison, flagging estimates above `h_star`.
///
/// `None` is a dead end, for `h` and `h_star` alike.
fn report(name: &str, h: Option<u64>, h_star: Option<u64>) {
    let overestimates = match (h, h_star) {
        (Some(h), Some(opt)) => h > opt,
        (Some(_), None) => false,
        (None, h_star) => h_star.is_some(),
    };
    let flag = if overestimates { "  inadmissible" } else { "" };

    match h {
        Some(h) => println!("{name:<32} {h:>4}{flag}"),
        None => println!("{name:<32} dead end{flag}"),
    }
}
