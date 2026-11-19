//! Properties of saturated cost partitioning over projections of random tasks.
//!
//! Each property compares against the exact heuristic `h*` from
//! [`sas_plus::Task::perfect_heuristic`]. `None` means a dead end.

use cost_partition::{MaxOverOrders, saturated_cost_partitioning};
use proptest::prelude::*;
use sas_plus::Abstraction;

use crate::strategies::{Case, case};

/// Checks that `h` never exceeds `h_star` and never claims a false dead end.
///
/// Returns a test-case error naming `what` when `h` is inadmissible.
fn check_admissible(what: &str, h: Option<u64>, h_star: Option<u64>) -> Result<(), TestCaseError> {
    match (h, h_star) {
        (Some(h), Some(opt)) => prop_assert!(h <= opt, "{what}: {h} > h* {opt}"),
        (None, h_star) => prop_assert_eq!(h_star, None, "{} claims a false dead end", what),
        (Some(_), None) => {}
    }

    Ok(())
}

/// `h*` of the case's initial state.
fn h_star(case: &Case) -> Option<u64> {
    case.task.perfect_heuristic()(&case.task.initial_state())
}

proptest! {
    /// SCP over a full order is admissible.
    #[test]
    fn scp_is_admissible(case in case()) {
        let scp = saturated_cost_partitioning(&case.projections, &case.order, &case.task.costs());

        check_admissible("SCP", scp.value(&case.task.initial_state()), h_star(&case))?;
    }

    /// Abstractions left out of the order get zero costs but still detect dead ends.
    #[test]
    fn scp_over_a_partial_order_is_admissible(case in case()) {
        let order = &case.order[..case.prefix];
        let scp = saturated_cost_partitioning(&case.projections, order, &case.task.costs());

        check_admissible("partial SCP", scp.value(&case.task.initial_state()), h_star(&case))?;
    }

    /// The cost functions sum to at most the original cost of each operator.
    #[test]
    fn cost_functions_partition_the_costs(case in case()) {
        let costs = case.task.costs();
        let scp = saturated_cost_partitioning(&case.projections, &case.order, &costs);

        for (label, &cost) in costs.iter().enumerate() {
            let shared: u64 = scp.cost_functions().iter().map(|cf| cf[label]).sum();
            prop_assert!(shared <= cost, "operator {label}: {shared} > {cost}");
        }
    }

    /// The first abstraction in the order keeps its full goal distance.
    #[test]
    fn scp_dominates_its_first_component(case in case()) {
        let costs = case.task.costs();
        let init = case.task.initial_state();
        let first = &case.projections[case.order[0]];
        let h_first = first
            .abstract_state(&init)
            .and_then(|s| first.lts().goal_distances(&costs)[s]);
        let h = saturated_cost_partitioning(&case.projections, &case.order, &costs).value(&init);

        if let (Some(h), Some(h_first)) = (h, h_first) {
            prop_assert!(h >= h_first, "SCP {h} below first component {h_first}");
        }
    }

    /// The maximum over orders is admissible and at least each of its components.
    #[test]
    fn max_over_orders_is_admissible_and_dominates(case in case()) {
        let costs = case.task.costs();
        let init = case.task.initial_state();
        let reversed: Vec<usize> = case.order.iter().rev().copied().collect();
        let max = MaxOverOrders::new(&case.projections, &[case.order.clone(), reversed], &costs);
        let h = max.value(&init);

        check_admissible("max over orders", h, h_star(&case))?;
        for component in max.heuristics() {
            if let (Some(h), Some(hc)) = (h, component.value(&init)) {
                prop_assert!(h >= hc, "max {h} below component {hc}");
            }
        }
    }
}
