# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog,
and this project adheres to Semantic Versioning.

## [Unreleased]

### Added
- Nothing yet.

## [0.0.2-alpha.0] - 2026-02-11

### Added
- Base `TimeScale` implementation with deterministic domain/pixel mapping.
- Base `PriceScale` implementation with inverted Y-axis mapping.
- Price autoscaling baseline from chart data (`PriceScale::from_data`).
- `ChartEngine` support for time/price mapping and runtime price autoscale.
- Extended unit/integration/property tests for time and price scales.

## [0.0.1-alpha.0] - 2026-02-11

### Added
- Initial project governance and architecture baseline.
- Modular crate scaffold for core/render/interaction/api/platform_gtk/extensions.
- Test harness with unit, integration, and property-test examples.
- Benchmark harness with criterion.
- GitHub Actions workflows for CI, security, and scheduled benchmarks.
- Initial repository bootstrap.
