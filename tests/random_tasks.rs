use cost_partition::{MaxOverOrders, saturated_cost_partitioning};
use sas_plus::random::{RandomTaskParams, Rng, random_task};
use sas_plus::{Abstraction, Projection};

#[test]
fn scp_over_projections_is_admissible_and_dominates_its_first_component() {
    let mut rng = Rng::new(7);
    let plain = RandomTaskParams::default();
    // Conditional effects and axioms: projections over-approximate both.
    let rich = RandomTaskParams {
        max_conditions: 2,
        num_derived: 2,
        ..plain
    };
    for (seed, params) in (0..200).flat_map(|s| [(s, plain), (s, rich)]) {
        let task = random_task(seed, params);
        let init = task.initial_state();
        let costs = task.costs();
        let n = task.num_variables();
        let mut patterns: Vec<Vec<usize>> = (0..n).map(|v| vec![v]).collect();
        patterns.extend((0..n).flat_map(|a| (a + 1..n).map(move |b| vec![a, b])));
        let projections: Vec<Projection> = patterns
            .into_iter()
            .map(|p| Projection::new(&task, p))
            .collect();

        let h_star = task.perfect_heuristic()(&init);
        let mut orders = Vec::new();
        for _ in 0..5 {
            let mut order: Vec<usize> = (0..projections.len()).collect();
            for i in (1..order.len()).rev() {
                order.swap(i, rng.range(0, i as u64) as usize);
            }
            let scp = saturated_cost_partitioning(&projections, &order, &costs);

            // Cost functions are a partition: they sum to at most the original costs.
            for (l, &c) in costs.iter().enumerate() {
                assert!(
                    scp.cost_functions().iter().map(|cf| cf[l]).sum::<u64>() <= c,
                    "seed {seed}"
                );
            }
            let first = &projections[order[0]];
            let h_first = first.lts().goal_distances(&costs)[first.abstract_state(&init).unwrap()];
            match (scp.value(&init), h_star) {
                (Some(h), Some(opt)) => {
                    assert!(h <= opt, "seed {seed}: SCP {h} > h* {opt}");
                    assert!(
                        h >= h_first.unwrap(),
                        "seed {seed}: SCP below first component"
                    );
                }
                (None, h) => assert_eq!(h, None, "seed {seed}: SCP claims a false dead end"),
                (Some(_), None) => {}
            }
            orders.push(order);
        }
        let max = MaxOverOrders::new(&projections, &orders, &costs);
        if let (Some(h), Some(opt)) = (max.value(&init), h_star) {
            assert!(h <= opt);
        }
    }
}
