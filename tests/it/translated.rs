//! SCP over atomic projections of Fast Downward translator output.
//!
//! The fixtures and their optimal costs come from sas-plus, which checks the
//! optima by exhaustive search (see `tests/fixtures/README.md`).

use cost_partition::{MaxOverOrders, saturated_cost_partitioning};
use sas_plus::Projection;
use sas_plus::fdr::SasFile;

/// Atomic projections of the `gripper-costs` fixture.
fn gripper_costs() -> (sas_plus::Task, Vec<Projection>) {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/gripper-costs/output.sas"
    );
    let task = SasFile::parse(&std::fs::read_to_string(path).unwrap())
        .unwrap()
        .to_task();
    let projections = (0..task.num_variables())
        .map(|v| Projection::new(&task, vec![v]))
        .collect();

    (task, projections)
}

/// A repeated index would spend its costs twice and drop its estimate.
#[test]
#[should_panic(expected = "saturated_cost_partitioning: abstraction 0 appears twice in order")]
fn scp_rejects_a_repeated_index() {
    let (task, projections) = gripper_costs();
    let _ = saturated_cost_partitioning(&projections, &[0, 0], &task.costs());
}

/// With no orders, the maximum would be 0 everywhere and miss dead ends.
#[test]
#[should_panic(expected = "MaxOverOrders::new needs at least one order")]
fn max_over_orders_rejects_no_orders() {
    let (task, projections) = gripper_costs();
    let _ = MaxOverOrders::new(&projections, &[], &task.costs());
}

/// SCP never exceeds the known optimal cost, with and without conditional
/// effects and axioms.
#[test]
fn scp_is_admissible_on_translator_output() {
    for (fixture, optimum) in [
        ("gripper-costs", 7),
        ("translated/sussman-anomaly", 3),
        ("translated/gripper-2", 3),
        ("translated/gripper-4", 9),
        ("translated/courier", 16),
        ("translated/wiring", 4),
    ] {
        let path = format!(
            "{}/tests/fixtures/{fixture}/output.sas",
            env!("CARGO_MANIFEST_DIR")
        );
        let task = SasFile::parse(&std::fs::read_to_string(path).unwrap())
            .unwrap()
            .to_task();
        let n = task.num_variables();
        let projections: Vec<Projection> =
            (0..n).map(|v| Projection::new(&task, vec![v])).collect();
        let order: Vec<usize> = (0..n).collect();
        let h = saturated_cost_partitioning(&projections, &order, &task.costs())
            .value(&task.initial_state())
            .unwrap();
        assert!(h <= optimum, "{fixture}: SCP {h} > h* {optimum}");
        if fixture == "gripper-costs" {
            assert!(h >= 1, "{fixture}: SCP is uninformed");
        }
        println!("{fixture}: SCP over atomic projections = {h} (h* = {optimum})");
    }
}
