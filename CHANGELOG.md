# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/sunsided/cost-partition/compare/v0.1.0...v0.2.0) - 2026-11-19

### Added

- Add the unsafe-forbidden badge to the README
- Add contribution guidelines
- Add git hooks and Copilot review rules
- Add a fuzz target for SCP on random tasks
- Add criterion benchmarks for building and evaluating SCP
- Add cargo-deny, Dependabot and typos configurations
- Add a Codecov configuration
- Add a Taskfile and split CI into one job per check

### Changed

- Name release PRs "Release vX.Y.Z"
- Update criterion to 0.8 and raise the MSRV to 1.86
- Ship the README example's fixture in the package
- Leave the fixtures' GPL license text out of the package
- Document every item and clear the pedantic findings
- Enable pedantic clippy, doc lints and forbid unsafe code
- Automate releases with release-plz and trusted publishing
- Expand the README and doctest its example
- Turn the random-task test into proptest properties
- Describe licensing with REUSE

### Fixed

- Correct the task ci and actions-lint descriptions
- Fix the major-bump regex matching body lines
- Fix the merge-queue branch check and the local fuzz target
- Fix the actionlint install and the fuzz build in CI

### Security

- Harden the publish workflow
