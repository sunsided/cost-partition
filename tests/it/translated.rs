//! SCP over atomic projections of Fast Downward translator output.
//!
//! The fixtures and their optimal costs come from sas-plus, which checks the
//! optima by exhaustive search (see `tests/fixtures/README.md`).

use cost_partition::saturated_cost_partitioning;
use sas_plus::Projection;
use sas_plus::fdr::SasFile;

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
