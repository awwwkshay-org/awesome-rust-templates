# Using this repository as a template

## Rename the starter

Search for these intentionally generic identifiers and replace them if useful:

- Cargo packages: `api`, `ui`, and `shared`
- Dioxus application name: `fullstack-mono-template`
- Database and Compose names: `app` and `app_test`
- Container image examples: `template-api` and `template-ui`

Keeping the short Cargo package names is also reasonable for a private monorepo.

## Replace the example feature

The Todo slice exists to prove the entire stack works. A clean replacement order
is:

1. Add the first real API contract to `packages/shared`.
2. Add or replace migrations in `apps/api/migrations`.
3. Implement the Axum route and SQLx query.
4. Add the typed server function in `apps/ui/src/server_functions.rs`.
5. Replace the Todo page with the first real screen.
6. Delete the Todo contracts, routes, tests, and migration after nothing uses them.

## Keep the template small

For a mostly solo project, add abstractions in response to actual pressure:

- Start with concrete SQLx functions; add repository traits when there is a second backend or testing need.
- Start with one API crate; split services only when deployment or ownership requires it.
- Put only HTTP contract types in `shared`; do not turn it into a general dumping ground.
- Keep the Dioxus server as a focused BFF; domain rules remain in the API.
- Add authentication after deciding whether the product needs sessions, tokens, or an external identity provider.
- Put repeatable maintenance commands in the root `scripts/` directory as they emerge.
