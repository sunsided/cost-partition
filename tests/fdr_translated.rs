use cost_partition::saturated_cost_partitioning;
use sas_plus::Projection;
use sas_plus::fdr::SasFile;

#[test]
fn scp_is_admissible_on_translator_output() {
    for (name, optimum) in [
        ("sussman-anomaly", 3),
        ("gripper-2", 3),
        ("gripper-4", 9),
        ("courier", 16),
        ("wiring", 4),
    ] {
        let path = format!(
            "{}/tests/fixtures/translated/{name}/output.sas",
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
        assert!(h <= optimum, "{name}: SCP {h} > h* {optimum}");
        println!("{name}: SCP over atomic projections = {h} (h* = {optimum})");
    }
}
