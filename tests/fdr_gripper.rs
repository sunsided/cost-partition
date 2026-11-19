use cost_partition::saturated_cost_partitioning;
use sas_plus::Projection;
use sas_plus::fdr::SasFile;

#[test]
fn scp_over_atomic_projections_is_admissible_on_gripper() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/gripper-costs/output.sas"
    );
    let task = SasFile::parse(&std::fs::read_to_string(path).unwrap())
        .unwrap()
        .to_task()
        .unwrap();
    let projections: Vec<Projection> = (0..task.num_variables())
        .map(|v| Projection::new(&task, vec![v]))
        .collect();
    let order: Vec<usize> = (0..projections.len()).collect();
    let h = saturated_cost_partitioning(&projections, &order, &task.costs())
        .value(&task.initial)
        .unwrap();
    assert!((1..=7).contains(&h), "h = {h}");
    println!("SCP over atomic projections: h(init) = {h} (h* = 7)");
}
