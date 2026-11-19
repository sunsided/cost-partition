# cost-partition

Cost partitioning over *any* `sas_plus::Abstraction` (projections, merge-and-shrink, ...): saturated cost partitioning for a given order, and the maximum over several orders.

**Tested:** SCP ≤ h*, SCP ≥ its first component, no false dead ends, and cost functions sum to at most the original costs, on random tasks and translator output, both with and without conditional effects and axioms (derived variables). `no_std` (needs `alloc`).

**Roadmap:** greedy and diversified orders, general (negative) costs, an optimal LP baseline (`good_lp`), uniform cost partitioning.

Reference: Seipp, Keller & Helmert (JAIR 2020).
