//! Properties of saturated cost partitioning over projections of random tasks.
//!
//! Each property compares against the exact heuristic `h*` from
//! [`sas_plus::Task::perfect_heuristic`]. `None` means a dead end.

use cost_partition::{MaxOverOrders, saturated_cost_partitioning};
use merge_and_shrink::{Bisimulation, LinearMerge, MergeAndShrink};
use proptest::prelude::*;
use sas_plus::Abstraction;
use sas_plus::random::{RandomTaskParams, random_task};

use crate::strategies::{Case, atomic_and_pair_projections, case};

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
    case.task.perfect_heuristic()(&case.initial_state())
}

proptest! {
    /// SCP over a full order is admissible.
    #[test]
    fn scp_is_admissible(case in case()) {
        let scp = saturated_cost_partitioning(&case.projections, &case.order, &case.task.costs());

        check_admissible("SCP", scp.value(&case.initial_state()), h_star(&case))?;
    }

    /// Abstractions left out of the order get zero costs but still detect dead ends.
    #[test]
    fn scp_over_a_partial_order_is_admissible(case in case()) {
        let order = &case.order[..case.prefix];
        let scp = saturated_cost_partitioning(&case.projections, order, &case.task.costs());

        check_admissible("partial SCP", scp.value(&case.initial_state()), h_star(&case))?;
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
        let init = case.initial_state();
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
        let init = case.initial_state();
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

    /// SCP stays admissible when projections and a merge-and-shrink abstraction
    /// share one order, with the merge-and-shrink abstraction at any position.
    ///
    /// Merge-and-shrink prunes dead-end abstract states, so `abstract_state`
    /// returns `None` for them: the dead-end path that projections never take.
    #[test]
    fn scp_with_merge_and_shrink_is_admissible(case in case()) {
        let num_vars = case.task.num_variables();
        let mas = MergeAndShrink::new(LinearMerge::in_variable_order(num_vars), Bisimulation)
            .build(&case.task);
        let mut abstractions: Vec<&dyn Abstraction> =
            case.projections.iter().map(|p| p as &dyn Abstraction).collect();
        abstractions.push(&mas);
        let mut order = case.order.clone();
        order.insert(case.prefix, abstractions.len() - 1);
        let costs = case.task.costs();
        let init = case.task.initial_state();

        let h_star = h_star(&case);

        let scp = saturated_cost_partitioning(&abstractions, &order, &costs);
        check_admissible("SCP with merge-and-shrink", scp.value(&init), h_star)?;
        // The reverse order puts merge-and-shrink at the mirrored position.
        let reversed: Vec<usize> = order.iter().rev().copied().collect();
        let max = MaxOverOrders::new(&abstractions, &[order, reversed], &costs);
        check_admissible("max with merge-and-shrink", max.value(&init), h_star)?;
    }
}

/// A state that merge-and-shrink prunes is a dead end for SCP and the maximum.
///
/// The random cases reach this path often, but by chance; this fixed seed
/// always does, and the first assert fails if it stops pruning.
#[test]
fn pruned_state_is_a_dead_end() {
    let task = random_task(1, RandomTaskParams::default());
    let mas = MergeAndShrink::new(
        LinearMerge::in_variable_order(task.num_variables()),
        Bisimulation,
    )
    .build(&task);
    let init = task.initial_state();
    assert_eq!(
        mas.abstract_state(&init),
        None,
        "seed 1 no longer prunes the initial state"
    );

    let projections = atomic_and_pair_projections(&task);
    let mut abstractions: Vec<&dyn Abstraction> =
        projections.iter().map(|p| p as &dyn Abstraction).collect();
    abstractions.push(&mas);
    let order: Vec<usize> = (0..abstractions.len()).collect();
    let costs = task.costs();

    assert_eq!(
        saturated_cost_partitioning(&abstractions, &order, &costs).value(&init),
        None
    );
    assert_eq!(
        MaxOverOrders::new(&abstractions, &[order], &costs).value(&init),
        None
    );
    assert_eq!(
        task.perfect_heuristic()(&init),
        None,
        "pruned, but h* finds a plan"
    );
}
