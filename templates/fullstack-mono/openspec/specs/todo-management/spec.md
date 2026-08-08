# Todo Management Specification

## Purpose

Define the observable behavior for creating, listing, completing, and removing
Todo items through the application.

## Requirements

### Requirement: Create a Todo

The system SHALL create an incomplete Todo from a non-empty title and SHALL
return the created item.

#### Scenario: Create with surrounding whitespace

- **WHEN** a client submits a title with non-whitespace content and surrounding whitespace
- **THEN** the system stores and returns the trimmed title with `completed` set to `false`

#### Scenario: Reject an empty title

- **WHEN** a client submits an empty or whitespace-only title
- **THEN** the system returns a bad-request response and creates no Todo

### Requirement: List Todos

The system SHALL return every current Todo in stable creation order.

#### Scenario: List existing Todos

- **WHEN** a client requests the Todo collection
- **THEN** the system returns the current Todos ordered by creation time

### Requirement: Toggle Todo completion

The system SHALL invert the completion state of an existing Todo.

#### Scenario: Toggle an existing Todo

- **WHEN** a client toggles an existing Todo
- **THEN** the system persists and returns the Todo with its completion state inverted

#### Scenario: Toggle an unknown Todo

- **WHEN** a client toggles a Todo identifier that does not exist
- **THEN** the system returns a not-found response

### Requirement: Delete a Todo

The system SHALL permanently remove an existing Todo.

#### Scenario: Delete an existing Todo

- **WHEN** a client deletes an existing Todo
- **THEN** the system returns a successful empty response and the Todo is no longer listed

#### Scenario: Delete an unknown Todo

- **WHEN** a client deletes a Todo identifier that does not exist
- **THEN** the system returns a not-found response

### Requirement: Render the initial Todo list

The web application SHALL render the initial Todo collection on the server and
SHALL remain interactive after browser hydration.

#### Scenario: Open the Todo application

- **WHEN** a web client requests the application
- **THEN** the initial Todo collection is present in server-rendered HTML
- **AND** the hydrated client can create, toggle, and delete Todos without a full-page reload
