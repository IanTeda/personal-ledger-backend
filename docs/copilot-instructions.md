# Copilot Instructions for Personal Ledger Backend

This file points Copilot to the repository-level guidance and the new `docs/copilot/` folder which contains detailed prompts, examples, and repo-specific context.

High-level plan
- Read `docs/copilot/config.md` to learn the repo context (languages, commands, devcontainer notes).
- Read `docs/copilot/rust.md` for Rust coding conventions and best practices.
- Use `docs/copilot/prompts.md` for suggested prompt templates to get focused help.
- Refer to `docs/copilot/examples.md` for concrete prompt → expected response examples.

Files you should check
- `docs/copilot/README.md` — overview of the folder and how to use it.
- `docs/copilot/config.md` — repository context: build/test commands, important files, devcontainer notes.
- `docs/copilot/rust.md` — Rust coding conventions, best practices, and quality checklist.
- `docs/copilot/prompts.md` — recommended prompt templates for common tasks.
- `docs/copilot/examples.md` — example prompts and expected responses.

Quick tips for Copilot in this repo
- Keep changes small and test-covered. Run `cargo test` after edits.
- Use `rustfmt` and `clippy` (available in the devcontainer) to format and lint changes.
- For docs, mdbooks is available on port `8001` and mdBook can be served with `mdbook serve --port 80001`.
- If the devcontainer is changed, rebuild with VS Code: `Dev Containers: Rebuild Container`.

If you'd like, enable a status script or VS Code task that runs smoke checks (build + tests) after Copilot applies edits; I can add that next.

---
Last updated: 2025-09-13
