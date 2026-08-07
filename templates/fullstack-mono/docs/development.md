# Development

## First-time setup

Install the stable Rust toolchain and Dioxus CLI:

```sh
rustup show
cargo install dioxus-cli@0.7.9 --locked
```

Start PostgreSQL and create the API environment file:

```sh
docker compose up -d postgres
cp apps/api/.env.example apps/api/.env
```

The API embeds SQL migrations and applies them during startup. You do not need
to install the SQLx CLI for normal development.

## Development loop

Run the API from the repository root:

```sh
cargo run -p api
```

Run the Dioxus SSR server and web client with hot reload in another terminal:

```sh
cd apps/ui
dx serve --web
```

The default addresses are:

| Service | Address |
| --- | --- |
| UI dev server | `http://localhost:8080` |
| API | `http://localhost:3001` |
| PostgreSQL | `postgres://postgres:postgres@localhost:5432/app` |

## Tests and checks

With a local PostgreSQL instance running and `DATABASE_URL` set:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p ui --target wasm32-unknown-unknown
cargo check -p ui --no-default-features --features server
```

Or run the tests in containers with an ephemeral test database:

```sh
docker compose --profile test run --rm test
```

SQLx creates an isolated database for each `#[sqlx::test]`, applies migrations,
and removes it after the test. The configured Postgres user therefore needs
permission to create databases.

## Mobile and desktop

The UI crate supports `web`, `desktop`, `mobile`, and `server` features. Dioxus
builds a server alongside each full-stack client during development. For a
deployed native client, compile in the public Dioxus server address:

```sh
SERVER_URL=https://app.example.com dx bundle --android --release
SERVER_URL=https://app.example.com dx bundle --ios --release
SERVER_URL=https://app.example.com dx bundle --desktop --release
```

SSR is web-only. Native clients render locally, then call the same explicit
server-function routes on `SERVER_URL`.

## Adding a migration

Create a monotonically named SQL file under `apps/api/migrations`:

```text
YYYYMMDDHHMMSS_short_description.sql
```

Keep migrations forward-only once they have been shared. During very early
template customization, it is fine to replace the example Todo migration before
any environment depends on it.
