# Plan to Rename Library from `sez` to `sessrums`

This plan outlines the steps required to rename the `sez` library to `sessrums` across the codebase, tests, documentation, and configuration files.

**Steps:**

1.  **Update `Cargo.toml`:**
    *   Change the `name` field in the main `Cargo.toml` file from `"sez"` to `"sessrums"`.
2.  **Update Code References:**
    *   Use a search and replace operation to replace all occurrences of `sez::` with `sessrums::` in all `.rs` files within the `src/`, `examples/`, and `tests/` directories. This will fix import paths and any other code references.
3.  **Update Documentation References:**
    *   Use a search and replace operation to replace all occurrences of the literal string "sez" with "sessrums" in all `.md` files within the `docs/`, `README.md`, and `work/` directories. This will update mentions of the library name in text.
4.  **Regenerate Build Artifacts and Documentation:**
    *   Run `cargo build` or `cargo check` from the project root. This will automatically regenerate the `target/` directory, including `Cargo.lock`, `target/tests/trybuild/sessrums/Cargo.toml`, and the documentation in `target/doc/`, with the new library name.