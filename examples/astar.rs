//! A* search with and without a cost-partitioned heuristic.
//!
//! Solves a task optimally twice: once blind (`h = 0`, which is uniform-cost
//! search) and once guided by the maximum of saturated cost partitionings over
//! projections. Both find a plan of the same, optimal cost; the heuristic
//! search expands far fewer states.
//!
//! Meant for small tasks: both searches keep every reached state in memory, and
//! there is a projection for every pair of variables.
//!
//! ```sh
//! cargo run --release --example astar                    # translated gripper-4
//! cargo run --release --example astar -- path/to/output.sas
//! ```

use std::cmp::Reverse;
use std::collections::hash_map::Entry;
use std::collections::{BinaryHeap, HashMap};
use std::error::Error;

use cost_partition::MaxOverOrders;
use sas_plus::fdr::SasFile;
use sas_plus::{OpId, State, Task};

use common::{atomic_and_pair_projections, read, rotations};

#[path = "support/common.rs"]
mod common;

/// Translator output used when no path is given.
///
/// Lives in the repository only; the published package leaves it out.
const DEFAULT_TASK: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/translated/gripper-4/output.sas"
);

/// Number of orders the heuristic maximizes over.
const ORDERS: usize = 8;

/// The outcome of one search.
struct Solution {
    /// Operators from the initial state to a goal state; `None` if unsolvable.
    plan: Option<Vec<OpId>>,
    /// Total cost of `plan`; `None` if unsolvable.
    cost: Option<u64>,
    /// Number of states taken off the open list and expanded.
    expansions: usize,
}

/// A best-first search node: where we came from, and the cost so far.
struct Node {
    /// The predecessor state and the operator applied to it; `None` for the initial state.
    parent: Option<(State, OpId)>,
    /// Cheapest known cost from the initial state.
    g: u64,
}

/// Solves the task blind and with SCP, and prints both results.
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_TASK.into());
    let sas = SasFile::parse(&read(&path)?)?;
    let task = sas.to_task();
    let costs = task.costs();

    let projections = atomic_and_pair_projections(&task);
    let orders = rotations(projections.len(), ORDERS);
    let scp = MaxOverOrders::new(&projections, &orders, &costs);

    println!("{path}\n");
    let blind = astar(&task, |_| Some(0));
    report("blind (h = 0)", &blind);
    let guided = astar(&task, |state| scp.value(state));
    report(&format!("max of SCP over {} orders", orders.len()), &guided);
    assert_eq!(
        blind.cost, guided.cost,
        "blind and guided A* found different costs"
    );

    if let Some(plan) = &guided.plan {
        print!("\n{}", sas.format_plan(plan));
    }

    Ok(())
}

/// A* from the initial state of `task`, guided by the admissible `h`.
///
/// `h` returning `None` prunes the state as a dead end. With an admissible
/// `h`, the returned plan is optimal.
fn astar(task: &Task, h: impl Fn(&State) -> Option<u64>) -> Solution {
    let costs = task.costs();
    let init = task.initial_state();
    let mut nodes = HashMap::from([(init.clone(), Node { parent: None, g: 0 })]);
    let mut open = BinaryHeap::new();
    let mut expansions = 0;

    if let Some(h0) = h(&init) {
        open.push(Reverse((h0, 0, init.0.clone())));
    }

    // The open list holds the raw values: `State` has no `Ord`.
    while let Some(Reverse((_, g, values))) = open.pop() {
        let state = State(values);
        if g > nodes[&state].g {
            continue; // A cheaper path to this state was expanded already.
        }

        if task.is_goal(&state) {
            return Solution {
                plan: Some(extract_plan(&nodes, state)),
                cost: Some(g),
                expansions,
            };
        }

        expansions += 1;
        for op in task.applicable_operators(&state) {
            let next = task.apply(op, &state);
            let next_g = g + costs[op];
            if !improve(&mut nodes, &next, &state, op, next_g) {
                continue;
            }

            if let Some(h_next) = h(&next) {
                open.push(Reverse((next_g + h_next, next_g, next.0)));
            }
        }
    }

    Solution {
        plan: None,
        cost: None,
        expansions,
    }
}

/// Records reaching `next` from `state` via `op` at cost `g`, if that is cheaper.
///
/// Returns whether `next` needs to be (re-)opened.
fn improve(
    nodes: &mut HashMap<State, Node>,
    next: &State,
    state: &State,
    op: OpId,
    g: u64,
) -> bool {
    let node = Node {
        parent: Some((state.clone(), op)),
        g,
    };

    match nodes.entry(next.clone()) {
        Entry::Occupied(mut known) if g < known.get().g => {
            known.insert(node);
            true
        }
        Entry::Occupied(_) => false,
        Entry::Vacant(slot) => {
            slot.insert(node);
            true
        }
    }
}

/// Walks the parent links back from `goal` and returns the plan in order.
fn extract_plan(nodes: &HashMap<State, Node>, goal: State) -> Vec<OpId> {
    let mut plan = Vec::new();
    let mut state = goal;
    while let Some((parent, op)) = nodes[&state].parent.clone() {
        plan.push(op);
        state = parent;
    }

    plan.reverse();

    plan
}

/// Prints the plan cost and the number of expansions of one search.
fn report(name: &str, solution: &Solution) {
    let cost = solution
        .cost
        .map_or_else(|| "unsolvable".to_owned(), |cost| format!("cost {cost}"));
    println!(
        "{name:<28} {cost:<12} {:>6} expansions",
        solution.expansions
    );
}
