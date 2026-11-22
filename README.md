# cost-partition

[![crates.io](https://img.shields.io/crates/v/cost-partition.svg)](https://crates.io/crates/cost-partition)
[![docs.rs](https://img.shields.io/docsrs/cost-partition)](https://docs.rs/cost-partition)
[![CI](https://github.com/sunsided/cost-partition/actions/workflows/ci.yml/badge.svg)](https://github.com/sunsided/cost-partition/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/sunsided/cost-partition/graph/badge.svg)](https://codecov.io/gh/sunsided/cost-partition)
[![MSRV](https://img.shields.io/crates/msrv/cost-partition)](https://crates.io/crates/cost-partition)
[![License](https://img.shields.io/crates/l/cost-partition)](https://github.com/sunsided/cost-partition#license)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

Cost partitioning over *any* `sas_plus::Abstraction` (projections, [merge-and-shrink](https://crates.io/crates/merge-and-shrink), ...): saturated cost partitioning for a given order, and the maximum over several orders. Different kinds of abstractions can share one order as `Vec<&dyn Abstraction>`.

Splitting each operator's cost among several abstractions, so that the parts sum to at most the original cost, makes the *sum* of their heuristics admissible. The crate implements saturated cost partitioning after Seipp, Keller & Helmert (JAIR 2020) and its maximum over orders, is `no_std` with `alloc`, and is checked against exact goal distances on random tasks and real translator output.

## Who this is for

- Authors of optimal classical planners who have several abstraction heuristics (pattern databases, merge-and-shrink, Cartesian or their own) and want an admissible combination that is usually stronger than taking their maximum.
- Researchers comparing cost partitioning strategies: the saturated cost function of every abstraction is exposed (`cost_functions`), and any type that implements `sas_plus::Abstraction` can take part.
- Teams that plan on embedded hardware, since the crate is `no_std` and has a single dependency.
- For the abstractions themselves, use [sas-plus](https://github.com/sunsided/sas-plus) (projections) and [merge-and-shrink](https://github.com/sunsided/merge-and-shrink); for landmark cost partitioning, LM-cut or operator counting, [planning-heuristics](https://github.com/sunsided/planning-heuristics), which also wraps this crate's heuristics for its `Heuristic` trait.

Part of the [planning and search crates](https://sunsided.github.io/planning-and-search/); see also this crate's [overview page](https://sunsided.github.io/planning-and-search/crates/cost-partition/).

## What cost partitioning is

A planning heuristic estimates, for a state, the cost of the cheapest plan to a goal. Optimal search such as A* needs it to be *admissible*: it never estimates more than the true optimum `h*`.

An *abstraction* gives such a heuristic cheaply. It maps every state to a smaller abstract state space, for example a projection that keeps only some variables, and uses the goal distance there as the estimate. Each abstraction is admissible on its own, but usually weak, because it sees only part of the task.

Adding the estimates of several abstractions is not admissible in general. Each one charges the full cost of every operator it needs, so an operator that several abstractions need is paid for several times. On the translated gripper-4 task, the projections' estimates add up to 24, while the optimal plan costs 9.

Cost partitioning removes the double counting. Each operator's cost is split into shares, one per abstraction, so that the shares sum to at most the original cost. Each abstraction then computes its goal distances under its own shares only. Any plan pays every operator at most once in total across the abstractions, so the sum of their estimates is admissible again.

## How saturated cost partitioning works

Saturated cost partitioning (SCP) builds the shares greedily, one abstraction at a time, in a given order:

1. Compute the abstraction's goal distances under the costs that are still left.
2. Take its *saturated* cost function: for each operator, the least cost that keeps all of those goal distances. That is the largest drop in goal distance along any transition the operator labels. Operators that do not bring this abstraction closer to the goal get 0.
3. Subtract the saturated costs from what is left, and pass the rest on to the next abstraction.

An abstraction early in the order keeps its full estimate and takes only the costs it needs, so later abstractions still get the costs it has no use for. Abstractions left out of the order get no costs, but still report dead ends: states from which no plan exists.

A small example: operators `a` (cost 2) and `b` (cost 3), with the optimal plan `a, b`, so `h* = 5`. Abstraction `X` only notices `a`, and abstraction `Y` notices both.

```text
                  share of a   share of b   estimate
X, full costs          2            3           2
Y, full costs          2            3           5
sum, full costs                                 7   > h* = 5, not admissible

SCP in order X, Y
X                      2            0           2   b does not change X's distances
Y                      0            3           3   only the costs X left over
sum                    2            3           5   = h* = 5
```

Different orders give different estimates, and none is best for every state. `MaxOverOrders` computes SCP for several orders and takes the largest estimate, which is still admissible.

## Installation

```sh
cargo add cost-partition
```

Or in `Cargo.toml`:

```toml
[dependencies]
cost-partition = "0.2"
```

## Usage

Build projections of a task from Fast Downward's translator, partition the costs among them, and evaluate the heuristic:

```rust
use cost_partition::{MaxOverOrders, saturated_cost_partitioning};
use sas_plus::Projection;
use sas_plus::fdr::SasFile;

# let output_sas = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/gripper-costs/output.sas");
// `output_sas` is the path of a translator output file.
let task = SasFile::parse(&std::fs::read_to_string(output_sas)?)?.to_task();
let costs = task.costs();
let projections: Vec<Projection> = (0..task.num_variables())
    .map(|v| Projection::new(&task, vec![v]))
    .collect();

// Saturated cost partitioning in one order.
let order: Vec<usize> = (0..projections.len()).collect();
let scp = saturated_cost_partitioning(&projections, &order, &costs);
let h = scp.value(&task.initial_state());

// The maximum over several orders dominates each of them.
let reversed: Vec<usize> = order.iter().rev().copied().collect();
let max = MaxOverOrders::new(&projections, &[order, reversed], &costs);
assert!(max.value(&task.initial_state()) >= h);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`None` from `value` means a dead end: no plan exists from that state.

## Examples

Both are meant for small tasks. They run on the translated `gripper-4` task from this repository by default, or on the translator output passed as an argument. `task examples` runs both (`task examples -- path/to/output.sas` for another task).

[`heuristics`](https://github.com/sunsided/cost-partition/blob/main/examples/heuristics.rs) shows why costs are partitioned. Adding the projections' estimates with full costs overestimates the optimum, so the sum is not admissible. SCP splits the costs first: it stays admissible and is stronger than the best single projection. Its estimates for gripper-4:

```text
optimal plan cost (h*)              9
best single projection              2
sum, full costs                    24  inadmissible
SCP, one order                      4
max of SCP over 8 orders            4
```

[`astar`](https://github.com/sunsided/cost-partition/blob/main/examples/astar.rs) solves the task optimally with A*, blind and guided by SCP, and prints the plan. Both plans are optimal; the guided search expands fewer states. Its summary for gripper-4, before the plan:

```text
blind (h = 0)                cost 9          230 expansions
max of SCP over 8 orders     cost 9          178 expansions
```

## Tested

SCP ≤ h*, SCP ≥ its first component, no false dead ends, and cost functions sum to at most the original costs. These hold as proptest properties and a fuzz target on random tasks (the properties also with a merge-and-shrink abstraction in the order), and on translator output, both with and without conditional effects and axioms (derived variables).

## Features

`no_std` (needs `alloc`). The crate has no Cargo features. It depends on `sas-plus` without its default features.

## MSRV

Rust 1.86. Raising it is a minor-version change.

## Not yet implemented

Greedy and diversified orders, general (negative) costs, an optimal LP baseline (`good_lp`), uniform cost partitioning.

Reference: Seipp, Keller & Helmert, *Saturated Cost Partitioning for Optimal Classical Planning*, JAIR 2020.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSES/Apache-2.0.txt](https://github.com/sunsided/cost-partition/blob/main/LICENSES/Apache-2.0.txt))
- MIT license ([LICENSES/MIT.txt](https://github.com/sunsided/cost-partition/blob/main/LICENSES/MIT.txt))

at your option. Test fixtures derived from VHPOP are GPL-2.0-or-later and are not part of the published crate; see [`REUSE.toml`](https://github.com/sunsided/cost-partition/blob/main/REUSE.toml).

## Contributing

See [CONTRIBUTING.md](https://github.com/sunsided/cost-partition/blob/main/CONTRIBUTING.md).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
