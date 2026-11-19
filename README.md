# cost-partition

[![crates.io](https://img.shields.io/crates/v/cost-partition.svg)](https://crates.io/crates/cost-partition)
[![docs.rs](https://img.shields.io/docsrs/cost-partition)](https://docs.rs/cost-partition)
[![CI](https://github.com/sunsided/cost-partition/actions/workflows/ci.yml/badge.svg)](https://github.com/sunsided/cost-partition/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/sunsided/cost-partition/graph/badge.svg)](https://codecov.io/gh/sunsided/cost-partition)
[![MSRV](https://img.shields.io/crates/msrv/cost-partition)](https://crates.io/crates/cost-partition)
[![License](https://img.shields.io/crates/l/cost-partition)](https://github.com/sunsided/cost-partition#license)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

Cost partitioning over *any* `sas_plus::Abstraction` (projections, [merge-and-shrink](https://crates.io/crates/merge-and-shrink), ...): saturated cost partitioning for a given order, and the maximum over several orders. Different kinds of abstractions can share one order as `Vec<&dyn Abstraction>`.

Splitting each operator's cost among several abstractions, so that the parts sum to at most the original cost, makes the *sum* of their heuristics admissible.

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

## Roadmap

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
