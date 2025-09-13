# Repository Context (for Copilot)

This repository is following the principals of [Zero to Production](https://github.com/LukeMathWalker/zero-to-production), but is using gRPC instead of REST to server the API.

## Quick facts

- Language: Rust
- Build: `cargo build`
- Test: `cargo test`
- Code coverage: `cargo tarpaulin` (available in devcontainer)

## Frameworks

  - [tonic](https://github.com/hyperium/tonic)
  - [sqlx (Postgres)](https://github.com/launchbadge/sqlx)
  - [chrono](https://docs.rs/chrono/latest/chrono/)
  - [serde](https://serde.rs/)
  - jsonwebtoken
  - [thiserror](https://docs.rs/thiserror/latest/thiserror/)
  - [tracing](https://docs.rs/tracing/latest/tracing/)


## Important files

- `Cargo.toml` — Cargo manifest
- `src/main.rs`, `src/lib.rs` — main application
- `docs/` — project docs (mdBook)
- `.devcontainer/devcontainer.json` — devcontainer setup (includes mdbooks, protoc, clippy, rustfmt, cargo-tarpaulin)
- `build.rs` - Protoc build script/code
- `Makefile.toml` - Application make scripts


## Project Organization

- Use semantic versioning in `Cargo.toml`.
- Include comprehensive metadata: `description`, `license`, `repository`, `keywords`, `categories`.
- Use feature flags for optional functionality.
- Organize code into modules using `mod.rs` or named files.
- Keep `main.rs` or `lib.rs` minimal - move logic to modules.
- Prefer `secrecy::Secret` for secrets and tokens in memory.
- Use `thiserror::Error` for domain errors and map `sqlx::Error` to structured error variants
- Avoid `SELECT *` in SQL queries; list explicit columns
- Use Australian English when writting comments


## Filestructure

- `config/` - Application configuration files
- `docs/` - Project documentation
- `migrations/` - SQLX database migrations
- `protos/` - GPRC protobuf definitation files
- `src/database/things/[delete.rs, insert.rs, mod.rs, model.rs, read.rs, update.rs]` - SQLX database service files
- `src/domain/object.rs` - Application domain types including struct, new, parsing, intotypes and unit tests
- `src/services/endpoint` - GRPC service endpoints
- `src/utils/utility.rs` - Application general utilility functions and modules
- `src/config.rs` - Application configuration module
- `src/error.rs` - Application error types
- `src/lib.rs` - Application library module
- `src/main.rs` - Application main entry point
- `src/router.rs` - Application endpoint definitions
- `src/rpc.rs` - Tonic import module
- `src/startup.rs` - Tonic startup module and logic abstracted to allow for consistent server configuration for intergation tests
- `src/telemetry.rs` - Use telemetry for logging since we are going to be async


## Testing Guidance

- Add unit tests under `src/` for pure logic
- Add integration tests under `tests/` using `sqlx::test` for DB-backed
  behaviour
- Mock external dependencies where possible in unit tests
- Use deterministic seeds for random/fake data in tests
- Use faker crate to generate random test data


## Security Guidance:

- Never commit secrets, keys, or environment files to the repo
- Use CI secrets or vaults for credentials
- Ensure JWT secrets and private keys are loaded from secure sources
- For sensitive information wrap the variable in `secrecy::Secret` to avoid leaking into logs and traces


## Commit Message Guidance

- style: "<area>: <short description>"
- examples:
    - "email-verification: add updated_at to model and migration"
    - "tests: add cursor pagination tests for email_verifications"


## Devcontainer Notes

- To rebuild devcontainer after changes: VS Code Command Palette → "Dev Containers: Rebuild Container".


## Local workflows

- To run the app: `cargo run`.


## Review Checklist Guidance

Suggested review checklist for AI-generated changes:
  - Does the change compile (cargo build)?
  - Are there new or updated tests, and do they pass?
  - Are DB migrations backward-compatible and idempotent?
  - Did you avoid leaking secrets into code or tests?
  - Are tracing/logging fields informative and non-sensitive?


## Tips for Copilot

- When changing code, prefer small, test-covered patches.
- If a change alters public APIs, update README and docs.
