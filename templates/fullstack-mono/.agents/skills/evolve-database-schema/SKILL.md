---
name: evolve-database-schema
description: Plan, implement, and validate PostgreSQL schema changes and their SQLx queries and tests. Use when adding or changing tables, columns, indexes, constraints, migrations, database-backed API behavior, or when diagnosing migration and SQLx test failures.
---

# Evolve the database schema

Make schema changes forward-only and keep migrations, SQLx row mappings,
queries, API contracts, and tests consistent.

## Workflow

1. Inspect all existing migrations and search for every query and Rust type that
   reads or writes the affected data.
2. Decide whether the change is additive, requires a backfill, or is destructive.
   Surface data-loss, locking, and rollout risks before implementing destructive
   work.
3. Add a monotonically named migration under `apps/api/migrations`:
   `YYYYMMDDHHMMSS_short_description.sql`.
4. For a new non-null column on populated data, add or derive a safe value before
   enforcing `NOT NULL`. Avoid table rewrites and long locks when a staged change
   is possible.
5. Add constraints for invariants the database must protect and indexes for
   demonstrated access paths. Do not add speculative indexes.
6. Update SQLx queries and `FromRow` mappings together. Select explicit columns
   instead of `SELECT *` so schema drift is visible.
7. Keep database rows internal to the API. Change `packages/shared` only when the
   serialized HTTP contract changes.
8. Add or update `#[sqlx::test(migrations = "./migrations")]` coverage. Test the
   behavior that depends on the new schema, not merely that the migration runs.
9. Run the database checks from
   `../validate-fullstack-project/SKILL.md`.

## Migration policy

- Do not edit, reorder, or delete a migration that may have run outside the local
  environment. Add a corrective migration instead.
- Replacing the starter migration is acceptable only during initial template
  customization before any shared environment depends on it.
- The API embeds and applies pending migrations on startup. Keep migrations safe
  under concurrent application startup before scaling to multiple replicas.
- Never run destructive SQL against a non-test database without explicit user
  authorization and a verified backup or recovery plan.

## Test database requirement

`#[sqlx::test]` creates isolated databases. Point `DATABASE_URL` at an
administrative PostgreSQL database using a role allowed to create databases.
The included Compose test profile provides this setup.
