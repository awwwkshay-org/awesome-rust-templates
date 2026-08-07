use dioxus::prelude::*;
#[cfg(feature = "server")]
use shared::CreateTodo;
use shared::Todo;
use uuid::Uuid;

#[get("/health")]
pub async fn health() -> Result<String> {
    Ok("ok".to_string())
}

#[get("/api/todos")]
pub async fn list_todos() -> Result<Vec<Todo>> {
    Ok(backend::send(backend::client().get(backend::url("/api/todos"))).await?)
}

#[post("/api/todos")]
pub async fn create_todo(title: String) -> Result<Todo> {
    Ok(backend::send(
        backend::client()
            .post(backend::url("/api/todos"))
            .json(&CreateTodo { title }),
    )
    .await?)
}

#[post("/api/todos/{id}/toggle")]
pub async fn toggle_todo(id: Uuid) -> Result<Todo> {
    Ok(
        backend::send(backend::client().patch(backend::url(&format!("/api/todos/{id}/toggle"))))
            .await?,
    )
}

#[delete("/api/todos/{id}")]
pub async fn delete_todo(id: Uuid) -> Result<()> {
    backend::send_empty(backend::client().delete(backend::url(&format!("/api/todos/{id}"))))
        .await?;
    Ok(())
}

#[cfg(feature = "server")]
mod backend {
    use dioxus::prelude::{ServerFnError, ServerFnResult};
    use serde::de::DeserializeOwned;
    use shared::ApiError;

    const DEFAULT_API_URL: &str = "http://localhost:3001";

    pub fn client() -> reqwest::Client {
        reqwest::Client::new()
    }

    pub fn url(path: &str) -> String {
        let base = std::env::var("API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string());
        format!("{}{path}", base.trim_end_matches('/'))
    }

    pub async fn send<T: DeserializeOwned>(request: reqwest::RequestBuilder) -> ServerFnResult<T> {
        let response = request.send().await.map_err(ServerFnError::new)?;
        let status = response.status();

        if status.is_success() {
            response.json().await.map_err(ServerFnError::new)
        } else {
            Err(api_error(response, status).await)
        }
    }

    pub async fn send_empty(request: reqwest::RequestBuilder) -> ServerFnResult {
        let response = request.send().await.map_err(ServerFnError::new)?;
        let status = response.status();

        if status.is_success() {
            Ok(())
        } else {
            Err(api_error(response, status).await)
        }
    }

    async fn api_error(response: reqwest::Response, status: reqwest::StatusCode) -> ServerFnError {
        let message = response
            .json::<ApiError>()
            .await
            .map(|error| error.message)
            .unwrap_or_else(|_| format!("API request failed with {status}"));

        ServerFnError::ServerError {
            message,
            code: status.as_u16(),
            details: None,
        }
    }
}
