---
name: implement-fullstack-feature
description: Implement or change an end-to-end feature across shared Rust contracts, the Axum API, PostgreSQL persistence, Dioxus server functions, and UI state. Use for new product behavior, extending or replacing the Todo example, adding API operations, or any change that crosses two or more application boundaries.
---

# Implement a full-stack feature

Deliver the smallest complete vertical slice. Preserve the dependency rules in
`docs/architecture.md` and follow an existing operation through the stack before
creating a new pattern.

## Workflow

1. Check `openspec/specs` and `openspec/changes` for the capability. For a new
   capability or user-visible behavior change, use the `openspec-propose` skill
   and obtain approval before implementation. If an approved change exists, use
   `openspec-apply-change` and treat its artifacts as the controlling scope.
2. Define the behavior, affected clients, failure cases, and acceptance checks.
3. Search for the closest existing slice and identify every affected boundary.
4. Add request and response types to `packages/shared` only when they cross the
   HTTP boundary. Keep database rows and internal state in the API.
5. If persistence changes, read
   `../evolve-database-schema/SKILL.md` before editing migrations or queries.
6. Implement the API route, handler, query, and error mapping. Validate input at
   the domain boundary and avoid exposing internal database errors.
7. Add or update `#[sqlx::test]` coverage for success and meaningful failure
   paths. Exercise the Axum router when HTTP status or serialization matters.
8. Add a typed Dioxus server function that forwards to the API. Keep `reqwest`
   and other server-only code behind `#[cfg(feature = "server")]`.
9. Update UI state for success and failure without assuming a request succeeds.
   Preserve SSR and hydration behavior for initial data.
10. Remove superseded example code only after references are gone. Use `rg` to
   check contracts, routes, labels, migrations, and documentation.
11. Read `../validate-fullstack-project/SKILL.md` and validate every changed
    boundary.

## Guardrails

- Do not import the API crate from the UI.
- Do not put handlers, SQLx types, or presentation state in `shared`.
- Keep the UI server a BFF; domain authorization and invariants belong in the
  API.
- Prefer explicit routes and concrete SQLx functions. Introduce traits or new
  crates only when the change demonstrates a real second implementation or
  ownership boundary.
- Update API and UI health behavior only when deployment checks are updated too.
- Update `docs/architecture.md` when a dependency or runtime boundary changes.
