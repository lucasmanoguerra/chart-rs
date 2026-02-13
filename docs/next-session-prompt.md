# Next Session Prompt

Use this prompt at the start of the next working session:

```text
Speak to me in Spanish, but keep all project artifacts in English (code, comments, commit messages, PR titles/bodies, technical docs).

Project context:
- Repo: chart-rs
- Goal: replicate Lightweight Charts v5.1 behavior in Rust for GTK4/Relm4 desktop usage.
- Current parity state (2026-02-13):
  - Checklist currently tracks parity items through `C-117` as `done` in `docs/parity-v5.1-checklist.md`.
  - Visual differential harness (`C-110`) and CI artifact workflow (`C-117`) are integrated.
  - Property stress guard (`C-111`) is integrated.
- Current active issue:
  - `cargo test-visual` fails on visual drift for 3 fixtures in `tests/lightweight_visual_differential_tests.rs`:
    - `lwc-style-line-basic`
    - `lwc-style-percentage-window`
    - `lwc-style-candles-log-axis-scale-price`
  - Reported diffs are:
    - `max_diff=230`, `mean_diff=1.71633671875`
    - `max_diff=230`, `mean_diff=1.7168935185185186`
    - `max_diff=230`, `mean_diff=1.4201541824196597`
- Test execution split (memory-aware):
  - Normal flow: `cargo test-core`
  - Heavy visual block: `cargo test-visual`
  - Config source: `.cargo/config.toml`
- GTK test status:
  - GTK adapter coverage is active in `src/platform_gtk/mod.rs` as a single stable unit test:
    `gtk_adapter_exposes_filled_slot_navigation_and_coordinate_policy_utilities`
  - Do not re-introduce ignored GTK tests unless there is a strict platform reason.

Execution policy:
1) Discover continuation point from:
   - docs/parity-v5.1-checklist.md
   - docs/v5.1-default-parity-audit.md
   - CHANGELOG.md
   - docs/developer-guide.md
   - docs/architecture.md
2) Decide the next work item automatically:
   - if there are pending parity IDs, pick the lowest valid ID first;
   - if there are no pending IDs, prioritize active regressions (currently visual drift in `cargo test-visual`).
3) Execute end-to-end in small batches (1-2 scoped items per PR), no half-finished work.
4) Keep work local by default; do not push/merge/release unless explicitly requested.

Block selection rules:
- Pick the lowest pending parity ID first when pending IDs exist.
- Respect dependencies: do not start a block that depends on a pending predecessor.
- If multiple candidates are valid, prioritize:
  1. highest parity impact vs Lightweight Charts,
  2. lowest regression risk,
  3. lowest coupling (modular changes first).

Definition of done for each block:
- Implementation complete.
- Tests updated (unit/integration/property where applicable).
- Criterion benches updated when performance-sensitive paths change.
- Docs updated:
  - docs/parity-v5.1-checklist.md
  - docs/developer-guide.md
  - docs/architecture.md
  - CHANGELOG.md
- Local validation green:
  - cargo fmt --all
  - cargo test-core
  - cargo test-visual
  - cargo clippy --all-targets --all-features -j 1 -- -D warnings
- If requested by user: commit/push/PR workflow; otherwise keep changes local.

Operational behavior:
- Do not stop for per-step confirmations; proceed autonomously.
- If something fails, diagnose, fix, and retry.
- Keep modular architecture and Unix-style separation of responsibilities.
- Every new feature must include enough tests.
- Keep memory usage constrained by default test settings in `.cargo/config.toml`.

Required report per closed batch:
- Closed block IDs.
- Local commit status (and PR links only if user explicitly asked for remote workflow).
- Files changed.
- Next top 3 candidates with rationale.
```
