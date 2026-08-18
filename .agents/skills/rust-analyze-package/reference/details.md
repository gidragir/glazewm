# Execution Pipeline (Strict Order)

1. **Recursive Analysis**: Traverse all `.rs` files within the target package recursively. Extract module trees, public APIs, trait implementations, macros, and inter-module data flow.
2. **Context Mapping**: Correlate package code with `[workspace.dependencies]` and `[workspace.package]` in the root `Cargo.toml`.
3. **Draft Construction (Internal)**: Build a technical context mapping based on findings in memory.
4. **Validation Phase**: Recursively verify the drafted context against the raw source code. Actively search for missing trait bounds, unlisted feature flags, undeclared workspace peers, or hallucinatory API endpoints. Correct any discrepancies.
5. **Final Output Generation & File Persistence**: Save the validated result directly as `README.md` in the target package folder (`[INSERT_PACKAGE_PATH]/README.md`).

# Strict Output Constraints

- Write the final technical documentation to `[INSERT_PACKAGE_PATH]/README.md`.
- Zero conversational text, introductions, or chain-of-thought metadata in the generated README.md.
- Maximize technical density. Use AST-like Markdown structures (nested lists, exact type signatures).

# Required README.md Structure

1. `## Component Role`: 1-2 sentences on exact workspace function.
2. `## Dependency Graph`:
   - Inherited (`workspace = true`).
   - External (specify enabled features).
   - Local workspace peers.
3. `## Public API`: Exact signatures of exported traits, structs, enums, and entry-point functions.
4. `## Architecture & Modules`: Recursive internal module hierarchy.
5. `## Execution Context`: Sync/async boundaries, runtime requirements (e.g., Tokio), state mutability, locking mechanisms.
