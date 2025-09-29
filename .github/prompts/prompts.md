# Recommended Prompts for Copilot

Use these templates to get reliable and scoped help from Copilot.

Fix a failing test:
```
Repo: Personal Ledger Backend (Rust)
Task: Fix failing test named `test_x` in `tests/`.
Files to inspect: `src/`, `Cargo.toml`, `tests/`.
Desired outcome: All tests pass under `cargo test`.
```

Add a small feature:
```
Repo: Personal Ledger Backend (Rust)
Task: Add [brief feature description].
Constraints: Keep changes minimal, add tests, update README.
Files to modify: list of target files or let Copilot suggest.
```

Make a refactor:
```
Repo: Personal Ledger Backend (Rust)
Task: Refactor [module/file] to improve [readability/performance].
Tests: Add/adjust unit tests if needed.
```

Ask for code explanation:
```
Repo: Personal Ledger Backend (Rust)
Task: Explain what the function `x` in `src/y.rs` does and its edge cases.
```

Quick tips:
- For DB-related code, include the `migrations/` SQL file snippets to ensure AI knows column names and constraints.
- Ask for both unit tests and integration tests when behavior touches the DB.
