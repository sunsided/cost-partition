//! Integration tests for cost partitioning, compiled as a single test binary.
//!
//! - [`translated`]: Fast Downward translator output with known optimal costs.
//! - [`properties`]: property tests over random tasks, checked against the exact
//!   heuristic `h*`.

mod properties;
mod strategies;
mod translated;
