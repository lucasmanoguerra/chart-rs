# Next Session Prompt

Use this prompt at the start of the next working session:

```text
Speak to me in Spanish, but keep all project artifacts in English (code, comments, commit messages, PR titles/bodies, technical docs).

Project context:
- Repo: chart-rs
- Goal: replicate Lightweight Charts v5.1 behavior in Rust for GTK4/Relm4 desktop usage.
- Current baseline: parity blocks R-001 through R-042 are completed.
- Last merged PR: #75.

Execution policy:
1) Discover continuation point from:
   - docs/parity-v5.1-checklist.md
   - CHANGELOG.md
   - docs/developer-guide.md
   - docs/architecture.md
2) Select next block(s) automatically and explain briefly why.
3) Execute end-to-end in small batches (1-2 blocks per PR), no half-finished work.

Block selection rules:
- Pick the lowest pending parity ID first.
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
  - cargo test --all-features -j 1
  - cargo clippy --all-targets --all-features -j 1 -- -D warnings
- Git workflow complete:
  - commit + push branch + PR + wait checks + squash merge + return to clean main

Operational behavior:
- Do not stop for per-step confirmations; proceed autonomously.
- If CI is pending, wait 1-2 minutes and poll again.
- If something fails, diagnose, fix, and retry.
- Keep modular architecture and Unix-style separation of responsibilities.
- Every new feature must include enough tests.

Required report per closed batch:
- Closed block IDs.
- Commits and PR links/status.
- Files changed.
- Next top 3 block candidates with rationale.
```

