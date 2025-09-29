# Repository Context (for Copilot)

This respository is the backend server for the 'Personal Ledger' project. 'Personal Ledger' aims to provide its users with an intuitive and user-friendly interface that offers insights into their spending habits and investment position. By presenting information clearly and concisely, users can effectively monitor their overall financial health, make informed decisions, and ultimately achieve their personal financial goals.

This repository should run as a deamon server that interfaces with a database and provides gRPC responses with the requested data.

This repository is following the principals of [Zero to Production](https://github.com/LukeMathWalker/zero-to-production), but is using gRPC instead of REST to server the API.

## Filestructure

- `config/` - Contains the server configuration files that are loaded at startup.
- `docs/` - Contains documentation for the project, API specifications and user guides.
- `.github/copilot-instructions/` - Contains copilot instructions for the project.
- `.github/copilot-prompts/` - Contains copilot prompts for the project.
- `migrations/` - Contains the SQLX database migrations.
- `protos/` - Contains the GPRC protobuf definitation files as a git submodule.
- `src/database/things/` - Contains the SQLX database service files for things, including seperate fils for delete, instert, model, read and update.
- `src/domain/object.rs` - Contains application domain custom types including struct, new, parsing, intotypes and unit tests.
- `src/services/endpoint` - Contains the GRPC endpoing services.
- `src/utils/utility.rs` - Contains the servers general utilility modules that are used across modules.
- `src/config.rs` - Contains the server configuration module.
- `src/error.rs` - Contains the server error types.
- `src/lib.rs` - Contains the server rust library module.
- `src/main.rs` - Contains the server main entry point.
- `src/router.rs` - Contains the server rpc api routes.
- `src/rpc.rs` - Contains the Tonic import module.
- `src/startup.rs` - Contains the Tonic startup module and logic abstracted to allow for consistent server setup for intergation tests with the actual server.
- `src/telemetry.rs` - Contains the configuration for using telemetry for logging since we are going to be async.


## Quick facts

- Language: Rust
- Build: `cargo build`
- Test: `cargo test`
- Code coverage: `cargo tarpaulin` (available in devcontainer)

## Frameworks

  - [tonic](https://github.com/hyperium/tonic)
  - [sqlx (SQLite and Postgres)](https://github.com/launchbadge/sqlx)
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
- Use Australian English when writing comments


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

