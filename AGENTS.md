# AGENTS.md

## Project

THTK-Studio is a desktop IDE for Touhou script modding.
Tech stack:

- Tauri v2
- Rust host/backend
- Vue 3 + TypeScript frontend
- The app targets script editing, project management, build integration, and preview tooling for formats such as ECL / ANM / MSG / STD.

## What matters

This project is not a generic text editor.
It is a domain-specific IDE.

When making changes, prioritize:

1. correctness of project workflow
2. maintainability of architecture
3. responsiveness of the desktop app
4. clear separation between frontend UI and Rust-side logic

Do not optimize for flashy UI at the cost of architecture.

## Architecture rules

- Keep heavy logic out of the frontend when possible.
- Parsing, indexing, preview computation, file scanning, and toolchain orchestration should prefer Rust-side implementation.
- Frontend should focus on editor UI, panels, state management, and visualization.
- Do not tightly couple Monaco/editor code with business logic.
- Prefer defining stable data structures between frontend and backend instead of passing ad-hoc blobs.

## Code style

- Prefer small, explicit modules.
- Avoid over-engineering.
- Do not introduce large abstractions unless they clearly reduce complexity.
- Keep function and type names descriptive and domain-accurate.
- Preserve existing naming unless there is a strong reason to refactor.

## Frontend rules

- Use Vue 3 + TypeScript.
- Prefer composition API.
- Keep components focused and not overly large.
- UI state and domain state should be separated.
- Do not put complex parsing or toolchain logic in Vue components.
- Editor-related features should be designed so they can later connect to language-service style backends.

## Rust / Tauri rules

- Prefer Rust for file operations, background tasks, indexing, cache management, and external tool integration.
- Minimize unsafe usage.
- Handle errors explicitly; do not swallow them.
- Long-running tasks should not block the UI.
- Shell/tool invocations should be wrapped cleanly and return structured results.

## Toolchain assumptions

- Existing Touhou community tools are part of the workflow.
- Do not remove or break compatibility with external build/decompile tools unless explicitly asked.
- Prefer wrapping external tools behind a stable internal interface.
- Paths, game versions, and tool arguments should not be hardcoded where avoidable.

## When changing features

Before implementing, first identify which layer the task belongs to:

- frontend view/panel
- editor integration
- Rust command/backend service
- toolchain wrapper
- parser / symbol / preview pipeline

Do not mix these layers casually.

## Expected workflow

For non-trivial tasks:

1. inspect relevant files first
2. make a short plan
3. implement minimal necessary changes
4. verify affected code paths
5. summarize what changed and any remaining risks

## Validation

When you finish a task:

- check for obvious type/build issues
- ensure imports and interfaces are consistent
- avoid leaving dead code or placeholder branches
- report any unverified assumptions clearly

## Do not

- do not rewrite large parts of the project unless necessary
- do not invent unsupported script semantics
- do not fake parser or preview correctness
- do not add new dependencies casually
- do not replace stable workflow decisions without explanation

## Output expectations

When responding after code changes, include:

- what changed
- which files were touched
- why this approach was chosen
- any follow-up work still needed