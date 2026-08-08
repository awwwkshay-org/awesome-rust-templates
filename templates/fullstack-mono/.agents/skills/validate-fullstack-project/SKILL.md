---
name: validate-fullstack-project
description: Validate changes to the Rust workspace, Axum API, SQLx migrations, Dioxus SSR and WebAssembly clients, and Docker images. Use before completing implementation, when reproducing CI failures, or when deciding which targeted and full-project checks a change requires.
---

# Validate the full-stack project

Run fast, targeted checks while iterating, then run the CI-equivalent checks for
every affected surface. Execute commands from the repository root unless a step
says otherwise.

## Select checks

- Always: formatting and focused tests for changed behavior.
- `apps/api`, `packages/shared`, or migrations: API/shared Clippy and tests with
  PostgreSQL.
- `apps/ui`: both the SSR server feature and WebAssembly web feature checks.
- Cargo manifests or lockfile: all Rust checks affected by the dependency.
- Dockerfiles, Compose, ports, health checks, or runtime configuration: validate
  Compose and build the affected image.
- Workflow changes: compare commands and environment with `.github/workflows`.

## CI-equivalent Rust checks

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --exclude ui --all-targets -- -D warnings
DATABASE_URL=postgres://postgres:postgres@localhost:5432/app cargo test --locked --workspace --exclude ui
cargo clippy --locked -p ui --no-default-features --features server --all-targets -- -D warnings
cargo clippy --locked -p ui --no-default-features --features web --target wasm32-unknown-unknown -- -D warnings
```

Start PostgreSQL first when it is not already available:

```sh
docker compose up -d postgres
```

Alternatively, run API/shared tests against the isolated Compose database:

```sh
docker compose --profile test run --rm test
```

Do not replace the separate Dioxus feature checks with a single default-feature
build; server and browser targets have different dependency and `cfg` surfaces.

## Container checks

Validate every maintained profile after Compose changes:

```sh
docker compose --profile apps config --quiet
docker compose --profile test config --quiet
docker compose --profile app_deps config --quiet
docker compose --profile full config --quiet
```

Build only the affected image while iterating, then both images for shared Cargo
or workspace changes:

```sh
docker build -f apps/api/Dockerfile -t app-api-check .
docker build -f apps/ui/Dockerfile -t app-ui-check .
```

## Handoff

Report which checks passed. List skipped checks with the concrete reason, such as
a missing PostgreSQL service, Docker daemon, target toolchain, or mobile SDK.
Never describe an unexecuted check as passing.
