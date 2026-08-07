# Rust full-stack template

A deliberately small Rust monorepo for starting a full-stack application:

| Path | Purpose |
| --- | --- |
| `apps/api` | Axum HTTP API |
| `apps/ui` | Dioxus full-stack UI with its own SSR server and mobile clients |
| `packages/shared` | Types shared by the API and UI |

The Todo feature is an example vertical slice, not an architecture you must keep.
It uses PostgreSQL through SQLx and an embedded migration.

## Prerequisites

- Rust stable
- The Dioxus CLI 0.7: `cargo install dioxus-cli@0.7.9 --locked`
- PostgreSQL, or Docker for the included local stack

The included toolchain file installs the WebAssembly target automatically.

## Run locally

Start PostgreSQL:

```sh
docker compose up -d postgres
```

Start the API (it runs pending migrations on startup):

```sh
cp apps/api/.env.example apps/api/.env
cargo run -p api
```

In a second terminal, start the UI server and web client together:

```sh
cd apps/ui
dx serve --web
```

Open <http://localhost:8080>. The first response is rendered by the Dioxus UI
server and then hydrated in the browser. The API listens separately on
<http://localhost:3001>.

The UI server reads the backend API address at runtime:

```sh
API_URL=https://api.example.com dx serve --web
```

For native development, set the public URL of the Dioxus UI server. Native apps
call its server-function endpoints but do not use SSR:

```sh
SERVER_URL=http://192.168.1.10:8080 dx serve --android
SERVER_URL=http://192.168.1.10:8080 dx serve --ios
```

## Run with Docker

Build and start PostgreSQL, the API, and the web UI:

```sh
docker compose up --build
```

Open <http://localhost:8080>. The Dioxus server renders HTML, serves the hydrated
web client, and exposes same-origin server functions that call the API service.

Run the full test suite in Docker against a dedicated test database:

```sh
docker compose --profile test run --rm test
```

## Quality checks

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --exclude ui --all-targets -- -D warnings
DATABASE_URL=postgres://postgres:postgres@localhost:5432/app cargo test --locked --workspace --exclude ui
cargo clippy --locked -p ui --no-default-features --features server --all-targets -- -D warnings
cargo clippy --locked -p ui --no-default-features --features web --target wasm32-unknown-unknown -- -D warnings
```

## Turn this into a new app

1. Rename the package names if desired.
2. Replace the Todo types and routes with your first real feature.
3. Replace the example migration with your first schema before publishing a new app.
4. Keep transport-safe request/response types in `packages/shared` so the UI server, native clients, and API stay aligned.
5. Add authentication, tracing exporters, and deployment manifests when the app requires them.

## Documentation

- [`docs/architecture.md`](docs/architecture.md): boundaries and dependency rules
- [`docs/development.md`](docs/development.md): local setup, checks, and migrations
- [`docs/deployment.md`](docs/deployment.md): images, Compose, and production notes
- [`docs/ci-cd.md`](docs/ci-cd.md): CI checks, GHCR publishing, and optional GitOps delivery
- [`docs/template-guide.md`](docs/template-guide.md): renaming and replacing the Todo slice

The root `scripts/` directory is intentionally empty (apart from `.gitkeep`) and
reserved for project automation added later.
