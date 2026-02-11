# Contributing

Thanks for contributing to `chart-rs`.

## Ground Rules

- Follow `AGENTS.md` as the architecture and quality contract.
- Read `docs/developer-guide.md` before implementing non-trivial features.
- Keep modules focused and responsibilities separated.
- Prefer small, reviewable pull requests.
- Every behavior change must include tests.

## Development Workflow

1. Create a branch from `main`.
2. Implement one coherent change.
3. Add or update tests.
4. Run local checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

5. Update docs/changelog as needed.
6. Open a pull request with a parity impact summary.

## Commit Guidance

- Use clear, imperative commit messages.
- Reference parity items when relevant.
- Keep unrelated changes in separate commits.

## Versioning and Releases

- Versioning follows SemVer.
- Changelog follows Keep a Changelog.
- Breaking changes require explicit migration notes.
