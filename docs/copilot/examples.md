# Example Prompts & Expected Responses


## Example 1 — Add a new API endpoint

Prompt:
```
Add a new POST endpoint `/transactions` that accepts JSON with fields `{date, amount, description}` and stores it in the database. Keep changes minimal and add a unit test.
```
Expected response outline:
- Modify `src/routes.rs` (or appropriate file) to add handler.
- Update data model in `src/models.rs`.
- Add a test in `tests/` that posts sample JSON and asserts a successful insert.
- Run `cargo test` and ensure green.


## Example 2 — Fix build error

Prompt:
```
Fix the build error: unresolved import `crate::db::Pool` in `src/service.rs`.
```
Expected response outline:
- Search for `db::Pool` definition, adjust imports or rename symbol.
- Provide a patch with `apply_patch` edits.
- Explain why the error happened and how tests are affected.
