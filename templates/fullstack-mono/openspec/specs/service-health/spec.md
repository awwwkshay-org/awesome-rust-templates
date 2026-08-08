# Service Health Specification

## Purpose

Define health behavior used by local development, orchestration, and deployment
checks.

## Requirements

### Requirement: API health reflects database availability

The API health endpoint SHALL report success only when the API can communicate
with PostgreSQL.

#### Scenario: Database is available

- **WHEN** a client requests the API health endpoint and PostgreSQL responds
- **THEN** the API returns a successful response containing `ok`

#### Scenario: Database is unavailable

- **WHEN** a client requests the API health endpoint and PostgreSQL cannot be queried
- **THEN** the API returns a server-error response

### Requirement: UI server health is available

The UI server SHALL expose a health endpoint for orchestration checks.

#### Scenario: UI server is running

- **WHEN** a client requests the UI server health endpoint
- **THEN** the UI server returns a successful response containing `ok`
