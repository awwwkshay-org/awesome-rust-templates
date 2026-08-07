use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, patch},
};
use shared::{ApiError, CreateTodo, Todo};
use sqlx::{FromRow, PgPool};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/todos", get(list_todos).post(create_todo))
        .route("/api/todos/{id}/toggle", patch(toggle_todo))
        .route("/api/todos/{id}", delete(delete_todo))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

async fn health(State(state): State<AppState>) -> Result<&'static str, AppError> {
    sqlx::query("SELECT 1").execute(&state.db).await?;
    Ok("ok")
}

#[derive(FromRow)]
struct TodoRow {
    id: Uuid,
    title: String,
    completed: bool,
}

impl From<TodoRow> for Todo {
    fn from(row: TodoRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            completed: row.completed,
        }
    }
}

async fn list_todos(State(state): State<AppState>) -> Result<Json<Vec<Todo>>, AppError> {
    let rows = sqlx::query_as::<_, TodoRow>(
        "SELECT id, title, completed FROM todos ORDER BY created_at, id",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows.into_iter().map(Todo::from).collect()))
}

async fn create_todo(
    State(state): State<AppState>,
    Json(input): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(AppError::bad_request("title cannot be empty"));
    }

    let row = sqlx::query_as::<_, TodoRow>(
        "INSERT INTO todos (id, title) VALUES ($1, $2) RETURNING id, title, completed",
    )
    .bind(Uuid::new_v4())
    .bind(title)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

async fn toggle_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Todo>, AppError> {
    let row = sqlx::query_as::<_, TodoRow>(
        "UPDATE todos SET completed = NOT completed WHERE id = $1 RETURNING id, title, completed",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::not_found("todo not found"))?;

    Ok(Json(row.into()))
}

async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    match result.rows_affected() {
        0 => Err(AppError::not_found("todo not found")),
        _ => Ok(StatusCode::NO_CONTENT),
    }
}

pub struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "database request failed");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "internal server error".into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiError {
                message: self.message,
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use shared::{CreateTodo, Todo};
    use sqlx::PgPool;
    use tower::ServiceExt;

    use super::{AppState, app};

    #[sqlx::test(migrations = "./migrations")]
    async fn creates_and_lists_a_todo(pool: PgPool) {
        let app = app(AppState::new(pool));
        let request = Request::post("/api/todos")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&CreateTodo {
                    title: "Ship it".into(),
                })
                .unwrap(),
            ))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .oneshot(Request::get("/api/todos").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let todos: Vec<Todo> = serde_json::from_slice(&body).unwrap();

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "Ship it");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn rejects_an_empty_title(pool: PgPool) {
        let request = Request::post("/api/todos")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"title":"   "}"#))
            .unwrap();

        let response = app(AppState::new(pool)).oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
