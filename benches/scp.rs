//! Benchmarks for building and evaluating cost-partitioned heuristics.
//!
//! The input is the translated `gripper-4` fixture with projections onto every
//! variable and every pair of variables.

use std::hint::black_box;

use cost_partition::{MaxOverOrders, saturated_cost_partitioning};
use criterion::Criterion;
use sas_plus::fdr::SasFile;
use sas_plus::{Projection, Task};

/// Number of orders [`MaxOverOrders`] maximizes over.
const ORDERS: usize = 8;

/// Reads the translated `gripper-4` fixture.
fn gripper_4() -> Task {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/translated/gripper-4/output.sas"
    );
    let text = std::fs::read_to_string(path).expect("fixture is readable");

    SasFile::parse(&text).expect("fixture parses").to_task()
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

/// `count` distinct orders over `len` abstractions: rotations of the identity.
fn rotations(len: usize, count: usize) -> Vec<Vec<usize>> {
    (0..count)
        .map(|shift| (0..len).map(|i| (i + shift) % len).collect())
        .collect()
}

/// Times building SCP for one order, building the maximum over several orders,
/// and evaluating that maximum on the initial state.
fn bench_scp(criterion: &mut Criterion) {
    let task = gripper_4();
    let costs = task.costs();
    let projections = atomic_and_pair_projections(&task);
    let orders = rotations(projections.len(), ORDERS);
    let init = task.initial_state();

    criterion.bench_function("scp/build", |b| {
        b.iter(|| saturated_cost_partitioning(black_box(&projections), &orders[0], &costs));
    });
    criterion.bench_function("max_over_orders/build", |b| {
        b.iter(|| MaxOverOrders::new(black_box(&projections), &orders, &costs));
    });

    let max = MaxOverOrders::new(&projections, &orders, &costs);
    criterion.bench_function("max_over_orders/value", |b| {
        b.iter(|| max.value(black_box(&init)));
    });
}

/// Runs every benchmark in this file with criterion's command-line flags.
fn main() {
    let mut criterion = Criterion::default().configure_from_args();

    bench_scp(&mut criterion);
    criterion.final_summary();
}
