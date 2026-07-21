# AI / agent rules (persistent context)

**Phase 1 — Build, Code, and Continuous Context Enforcement**

These rules are the source of truth for every agent working in this repository. On every Phase 1 execution, re-verify this file exists at the repository root and still matches this intent.

## Language and design

1. **Code strictly from first principles using only Rust.** Prefer the compiler’s type system to eliminate entire classes of errors (enums over stringly state, `Result`/`Option` over sentinel values, newtypes when they clarify domain boundaries).
2. Do not introduce non-Rust application languages for core product logic. Embedded HTML/JS/CSS for the dashboard is allowed only as data/strings generated from Rust (e.g. maud / `const` fragments), not as a parallel app stack.

## Cognitive load and file structure

3. **Strict 250-line limit per `.rs` file** (production and tests). If a file would exceed 250 lines, split it before landing the change.
4. **Split exclusively at logical function / domain boundaries** — not arbitrary mid-function cuts.
5. **Explicit, domain-specific file naming** (e.g. `force_played_pair.rs`, `poster_proxy.rs`, `connection_test.rs`). Avoid generic names like `utils2.rs` or `misc.rs`.

## Quality bar (must pass locally)

6. **Eliminate dead code** (unused modules, stubs, empty re-exports, `assert!(true)` placeholder tests).
7. Strictly pass:
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo audit`
   - `cargo +nightly udeps --all-targets` (or equivalent nightly udeps invocation)

## Testing strategy

8. **Maximize testing leverage** on:
   - **Integration tests** and **core logic** (matching, sync, config, force-skip, UserData).
9. **Reserve end-to-end tests** for critical user paths only (dashboard smoke, force start, auth-free health).

## Phase gates

10. When Phase 1 work is **locally verified** (fmt, clippy, audit, udeps, tests, line limits), **halt**.
11. **Do not start Agent Review** until the user explicitly activates the Agent Review Phase.
12. Product docs: prefer a **Blue Ocean README** (one-line install + one perfect example); keep architecture in secondary docs under `docs/`.

## Product domain (StateSync)

- StateSync syncs **watched**, **resume**, and **favorites** across Emby/Jellyfin (and same-type pairs). It does not move media files.
- Prefer honest UI: no controls that do nothing. Prefer skip-if-equal and clear storytelling over silent rewrites.

## Enforcement checklist (every Phase 1 run)

- [ ] This file (`ai-rules.md`) is present at repo root and current
- [ ] No production `.rs` file exceeds 250 lines
- [ ] No test `.rs` file exceeds 250 lines
- [ ] `cargo fmt` / `clippy -D warnings` / `audit` / `udeps` pass
- [ ] Core/lib tests pass
- [ ] Stop and await Agent Review Phase activation
