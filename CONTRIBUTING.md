# Contributing to Brio

Thank you for your interest in contributing to Brio! We are building a high-security Micro-Kernel for AI orchestration.

## Prerequisites

- **Rust**: Latest stable version (`rustup update stable`).
- **Wasm Target**: `rustup target add wasm32-wasi` and `wasm32-unknown-unknown`.
- **Wasm Tools**: We recommend installing `wasm-tools` for inspecting binary components.
- **GitHub CLI**: `gh` for managing issues and PRs.

## Workflow

1. **Fork & Clone**: Fork the repo and clone it locally.
2. **Branch**: Create a branch for your feature or fix.
   ```bash
   git checkout -b feature/my-new-feature
   ```
3. **Implement**: Write your code.
   - Follow the "Lean Kernel" philosophy.
   - Keep dependencies minimal.
   - Use strict typing and `WIT` definitions.
4. **Verify**:
   ```bash
   cd brio-core
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test --workspace
   ```
5. **Commit**: Use [Conventional Commits](https://www.conventionalcommits.org/).
   The commit contains the following structural elements, to communicate intent to the consumers of your library:

   - **fix**: a commit of the type fix patches a bug in your codebase (this correlates with PATCH in Semantic Versioning).
   - **feat**: a commit of the type feat introduces a new feature to the codebase (this correlates with MINOR in Semantic Versioning).
   - **BREAKING CHANGE**: a commit that has a footer BREAKING CHANGE:, or appends a ! after the type/scope, introduces a breaking API change (correlating with MAJOR in Semantic Versioning). A BREAKING CHANGE can be part of commits of any type.
   - types other than fix: and feat: are allowed, for example @commitlint/config-conventional (based on the Angular convention) recommends build:, chore:, ci:, docs:, style:, refactor:, perf:, test:, and others.
   - footers other than BREAKING CHANGE: <description> may be provided and follow a convention similar to git trailer format.

   Additional types are not mandated by the Conventional Commits specification, and have no implicit effect in Semantic Versioning (unless they include a BREAKING CHANGE). A scope may be provided to a commit’s type, to provide additional contextual information and is contained within parenthesis, e.g., feat(parser): add ability to parse arrays.

6. **Pull Request**: Open a PR against `main`. Link the relevant Issue.

## Architecture Guidelines

- **Zero-Copy**: prefer passing references or using shared memory abstractions where possible.
- **Isolation**: Always assume the Guest component is untrusted.
- **State**: All durable state goes to SQLite or the Vector Store. NO local file persistence outside the isolated session workspace.

## License

By contributing, you agree that your contributions will be licensed under the project's [LICENSE](./LICENSE).
