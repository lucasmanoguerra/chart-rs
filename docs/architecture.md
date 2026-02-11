# Architecture Overview

`chart-rs` follows a modular architecture with strict dependency direction.

## Layers

- `core`
  - pure domain model
  - scale math and deterministic transformations
- `interaction`
  - event/state transitions
  - chart intent orchestration
- `render`
  - render contracts
  - backend implementations
- `api`
  - Rust-idiomatic public interface
- `platform_gtk` (feature-gated)
  - GTK4/Relm4 adapter
- `extensions`
  - optional feature-gated advanced capabilities

## Dependency Rules

Allowed direction:
- `api` -> `core`, `interaction`, `render`
- `platform_gtk` -> `api`
- `extensions` -> `api`, `core`, `interaction`, `render`

Forbidden direction:
- `core` -> `platform_gtk`
- direct GTK coupling in `core`/`interaction`
- mixed responsibilities inside one module

## Testing Model

Each feature requires:
- unit tests for local invariants
- integration tests for cross-module behavior
- property tests with `proptest`
- benchmark updates with `criterion` for performance-sensitive paths
