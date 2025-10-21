use crate::{Config, Storage};
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};

#[derive(Clone)]
pub struct Server {
    config: Config,
    storage: Storage,
}

#[derive(Clone)]
struct AppState {
    config: Config,
    storage: Storage,
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
struct ListKeysResponse {
    keys: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0.downcast_ref::<std::io::Error>() {
            Some(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
            None => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
        };

        let body = Json(ErrorResponse {
            error: error_message,
        });

        (status, body).into_response()
    }
}

struct AppError(anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl Server {
    pub fn new(config: Config, storage: Storage) -> Self {
        Self { config, storage }
    }

    pub async fn run(self, addr: &str) -> Result<()> {
        let state = AppState {
            config: self.config,
            storage: self.storage,
        };

        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/data", get(list_keys_handler))
            .route("/data/:key", get(get_data_handler))
            .route("/data/:key", put(put_data_handler))
            .route("/data/:key", delete(delete_data_handler))
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
            .with_state(Arc::new(state));

        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("Server listening on {}", addr);

        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        Ok(())
    }
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn list_keys_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ListKeysResponse>, AppError> {
    let keys = state.storage.list_keys()?;
    Ok(Json(ListKeysResponse { keys }))
}

async fn get_data_handler(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Result<Response, AppError> {
    match state.storage.get(&key)? {
        Some(value) => Ok((StatusCode::OK, value).into_response()),
        None => Ok((StatusCode::NOT_FOUND, "Key not found").into_response()),
    }
}

async fn put_data_handler(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    body: bytes::Bytes,
) -> Result<StatusCode, AppError> {
    state.storage.put(&key, &body)?;
    Ok(StatusCode::CREATED)
}

async fn delete_data_handler(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Result<StatusCode, AppError> {
    let deleted = state.storage.delete(&key)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}
