# truth Vertical Timeline Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan in the current session. Preserve existing uncommitted layout and editor-view fixes.

**Goal:** Render entry-separated vertical timelines with nested loops, interrupt entry points and source navigation.

**Architecture:** Extend the existing Rust analysis response with a recursive timeline. Keep existing outline/reference fields compatible. Vue renders the structure and uses the current editor event bridge for navigation and highlighting.

**Tech Stack:** Existing Rust, serde, regex, Vue 3 and TypeScript; no new dependencies.

**Spec:** `docs/superpowers/specs/2026-11-09-truth-vertical-timeline-design.md`

## Global Constraints

- Keep parsing in Rust; do not change truth or introduce a VM.
- Retain source order, absolute/relative time spelling and original interrupt positions.
- Preserve the current uncommitted fixes and `.qoder/` files; do not commit or package unless requested.
- Browser verification is limited to static screenshots.

## Review Focus

- Braces and keywords inside strings/comments must not create blocks.
- Inline/multiline block headers and nested loop exits must retain correct source targets.
- Consecutive interrupt labels share an entry point without inventing a handler boundary.
- Times after loops/interrupts must not be presented as evaluated runtime times.
- Switching files while analysis is pending must not show the previous file's timeline.

### Task 1: Rust timeline structure and wire contract

**Files:** Create `src-tauri/src/modules/truth/timeline.rs`; modify `analysis.rs`, `source.rs`, `mod.rs`, `src/types/truth.ts`, and `src-tauri/src/wire_format_tests.rs`.

- [x] Add source-based tests for nested loop timing, interrupts, jump targets, and comment/string masking.
- [x] Implement recursive structural nodes with source ranges, original source text, timing groups and optional target lines; expose `ScriptAnalysis.timeline`.
- [x] Pin the serialized node and entry fields; mirror them in TypeScript.
- [x] Run relevant Rust tests using `tauri-env linux cargo test --manifest-path src-tauri/Cargo.toml --bin thtk-studio` with focused filters (the library target is only a mobile stub).

### Task 2: Vertical timeline UI

**Files:** Modify `TimelinePanel.vue`; create `TruthTimelineNodes.vue` and `tests/unit/truthTimeline.spec.ts`.

- [x] Replace flat events with entry sections and separators, nested timing groups, collapsible controls, interrupt badges and source previews.
- [x] Use the existing cursor/reveal events for highlighting and navigation; reset per-file fold state and ignore obsolete analysis results.
- [x] Verify actual component behavior for folding, navigation, highlighting and file replacement.

### Task 3: Concentrated verification

- [x] Run `npm run typecheck`, `npm run test`, and `npm run build`.
- [x] Review static screenshots at normal and narrow sidebar widths with nested loops and interrupts.
- [x] Review the diff for source-order/time correctness and report remaining limitations. Keep changes uncommitted.

## Completion evidence

- Rust `truth` filter: 15 passed, 2 pre-existing integration checks ignored.
- After review, the focused timeline tests passed all 4 cases, including do-while continue targeting the trailing condition.
- Full frontend suite: 140 passed; the 3 timeline behavior tests were rerun after the final navigation adjustment and passed.
- `npm run typecheck` and `npm run build` passed.
- Static screenshots at 320px and 200px widths reviewed using production Rust timeline output. No browser click automation or Windows runtime verification.
- Final read-only review found one do-while navigation issue, fixed in the existing targetLine field. No further blocking findings.
- Integrated with the sidebar layout fixes for the requested commit/push. No installer rebuilt.
