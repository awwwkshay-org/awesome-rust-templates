# Containers and deployment

## Images

Both Dockerfiles use the repository root as their build context because both
apps depend on `packages/shared`.

```sh
docker build -f apps/api/Dockerfile -t template-api .
docker build -f apps/ui/Dockerfile -t template-ui .
```

The API image expects `DATABASE_URL` at runtime. Optional settings are `HOST`,
`PORT`, and `RUST_LOG`.

The UI image contains the Dioxus SSR server executable and its web assets.
`API_URL` is runtime configuration for the UI server:

```sh
docker run --rm -p 8080:8080 \
  -e API_URL=https://api.example.com \
  template-ui
```

Web browsers call same-origin Dioxus server functions. Those functions forward
domain operations to `API_URL`, so the API service is not responsible for SSR,
frontend assets, or hydration.

## Local stack

```sh
docker compose up --build
```

Compose persists development data in the `postgres-data` volume. To stop the
stack without deleting data:

```sh
docker compose down
```

To intentionally remove the development database too:

```sh
docker compose down --volumes
```

## Production notes

- Supply secrets through the deployment platform, never through committed env files.
- Use managed PostgreSQL or back up the database volume.
- Terminate TLS at a load balancer or ingress.
- Point native clients at the public UI server using `SERVER_URL` during their build.
- Keep the domain API private to the UI server when no third-party client needs it.
- Add an explicit, allowlisted CORS policy only if browsers must call the domain API directly.
- Run at least two API replicas only after migrations are safe to execute concurrently.
- Deploy the digest-pinned images produced by the release workflow; see
  [`ci-cd.md`](ci-cd.md) for registry and GitOps configuration.
