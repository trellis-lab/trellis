---
description: Check Rust formatting and clippy lints, and fix any issues found.
model: haiku
allowed-tools: Bash, Read, Edit
---

Run `cargo fmt --check --all` and `cargo clippy --workspace` to check for Rust formatting and lint issues across the workspace.

## Part 1: Formatting

1. Run `cargo fmt --check --all` using the Bash tool
2. If the command exits with code 0 (no issues), report that all files are properly formatted
3. If formatting issues are found, the output will show diffs of what needs to change. For each file with issues:
   - Read the file
   - Apply the formatting fixes shown in the diff using the Edit tool
4. After fixing all issues, run `cargo fmt --check --all` again to confirm everything passes
5. Report a summary of what was fixed

## Part 2: Clippy

1. Run `cargo clippy --workspace` using the Bash tool
2. If the command exits with code 0 and no warnings or errors, report that clippy found no issues
3. If clippy reports **errors**, fix them immediately:
   - Read the affected file
   - Apply the fix suggested by clippy using the Edit tool
4. If clippy reports only **warnings**, list them and ask the user whether to fix them before proceeding
5. After fixing issues, run `cargo clippy --workspace` again to confirm everything passes
6. Report a summary of what was fixed
