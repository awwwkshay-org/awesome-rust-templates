# Architecture

This template favors a small number of clear deployables: a domain API and a
Dioxus full-stack UI server. PostgreSQL is the only supporting service.

```text
web browser ── SSR/hydration ──► apps/ui server ── HTTP/JSON ──► apps/api ──► PostgreSQL
native app  ── server functions ─────▲

apps/ui  ───────► packages/shared ◄────── apps/api
```

## Dependency rules

- `shared` contains serializable API contracts and small domain value types.
- `shared` must not depend on Axum, Dioxus, a database driver, or platform APIs.
- `api` owns domain HTTP routes, persistence, and database migrations.
- `ui` owns presentation plus an independent Dioxus server for SSR and server functions.
- The UI never imports the API crate. The shared contract is their only compile-time coupling.
- The UI server acts as a small backend-for-frontend (BFF); it forwards Todo operations to the API.

## Current vertical slice

The Todo example demonstrates the minimum useful flow:

```text
Dioxus page → typed server function → UI server → Axum API → SQLx/PostgreSQL
            ←────── shared Todo data ──────────────────────────
```

API routes (port 3001):

- `GET /health`
- `GET /api/todos`
- `POST /api/todos`
- `PATCH /api/todos/{id}/toggle`
- `DELETE /api/todos/{id}`

The UI server (port 8080) exposes matching server-function routes plus `/health`.
On web, `use_server_future` resolves Todo data before the initial HTML is sent
and Dioxus serializes that result for hydration. On mobile and desktop, SSR does
not apply; the same function call is sent to the configured UI server URL.

## Growing the template

Prefer feature modules over horizontal folders once the API grows. A feature
can own its handlers, service logic, repository, and tests. Put only types that
cross the HTTP boundary in `shared`; database rows and internal entities should
remain server-side.

`AppState` owns a cloneable SQLx connection pool. Queries stay inside the API,
and migrations are embedded in the binary and applied on startup. Start with
concrete query functions; introduce repository traits only when you have a real
second implementation or need a sharper testing boundary.

The containerized UI is the Dioxus-generated server executable plus its `public`
assets. It does not rely on the API binary to host or render the frontend. The UI
server needs the API only for domain data, just as a native client needs a remote
backend.
