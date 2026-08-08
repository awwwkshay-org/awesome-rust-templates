# Repository instructions for coding agents

## Project overview

This repository is a Rust full-stack monorepo with three boundaries:

- `apps/api`: Axum HTTP API, domain behavior, SQLx queries, and migrations.
- `apps/ui`: Dioxus SSR/web/native UI and a small server-function BFF.
- `packages/shared`: transport-safe types shared by the API and UI.

Read `docs/architecture.md` before changing boundaries and
`docs/development.md` before changing build or test workflows. Inspect the
package names in the Cargo manifests instead of assuming they have retained the
template defaults.

## Architecture rules

- Keep domain rules and persistence in the API.
- Keep presentation and API forwarding in the UI. The UI must not depend on the
  API crate.
- Put only serialized HTTP contracts and small domain value types in `shared`.
  Do not add Axum, Dioxus, SQLx, or platform-specific dependencies there.
- Treat a feature as a vertical slice: shared contract, migration when needed,
  API route/query, UI server function, UI state, and tests.
- Preserve separate Dioxus feature builds. Server-only dependencies must remain
  behind the `server` feature, and browser code must compile for
  `wasm32-unknown-unknown`.

## Agent Skills

Skills use the open `SKILL.md` format and live under `.agents/skills`. Load the
matching skill before starting one of these tasks, even when the current agent
does not discover repository skills automatically:

- Feature work: `.agents/skills/implement-fullstack-feature/SKILL.md`
- PostgreSQL schema or migration work:
  `.agents/skills/evolve-database-schema/SKILL.md`
- Testing, CI reproduction, or pre-handoff validation:
  `.agents/skills/validate-fullstack-project/SKILL.md`

OpenSpec workflow skills are also installed in `.agents/skills/openspec-*` and
are managed by the OpenSpec CLI. Do not hand-edit generated OpenSpec skills; run
`openspec update` after upgrading the CLI.

## Spec-driven development

Use the OpenSpec workflow for new capabilities, user-visible behavior changes,
breaking contract or schema changes, and significant architecture work:

1. Explore uncertain ideas with the `openspec-explore` skill when useful.
2. Create a proposal with `openspec-propose`. Review and approve its proposal,
   design, delta specs, and tasks before implementation.
3. Implement approved tasks with `openspec-apply-change`, loading the relevant
   project skill above for stack-specific guidance.
4. Keep artifacts aligned with discoveries by using `openspec-update-change`.
5. Run `openspec validate <change>` and the project validation skill.
6. Sync the accepted delta into `openspec/specs` and archive the completed change.

Existing behavior is documented in `openspec/specs`. Active work belongs in
`openspec/changes`; do not implement an active change whose planning artifacts
are incomplete or unapproved. Small fixes that restore already-specified
behavior may proceed directly, but must still be tested.

## Working conventions

- Use the pinned toolchain and committed `Cargo.lock`. Use `--locked` for
  validation commands.
- Keep migrations forward-only after they have been shared. Never edit or remove
  deployed migrations unless the user explicitly requests a recovery plan.
- Add focused tests for behavior changes. API database tests use `#[sqlx::test]`
  and require a PostgreSQL connection with permission to create databases.
- Never commit `.env` files, credentials, tokens, production data, or generated
  build output.
- Keep the template small. Add abstractions in response to a concrete need, not
  in anticipation of one.
- Update documentation and example environment values when commands,
  configuration, public contracts, or architecture change.

## Before finishing

Load `.agents/skills/validate-fullstack-project/SKILL.md` and run the checks
relevant to the changed surfaces. Always run formatting. Do not claim database,
WebAssembly, container, or mobile validation unless that check actually ran.
Report skipped checks and the reason.
