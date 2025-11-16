use anyhow::Result;
use axum::{
    Router,
    routing::{get, post, delete},
};
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::session::SessionManager;
use crate::config::Config;
use super::handlers;

pub type AppState = Arc<RwLock<SessionManager>>;

pub async fn run(
    bind_addr: String,
    session_manager: Arc<RwLock<SessionManager>>,
    config: Config,
) -> Result<()> {
    let app = Router::new()
        // WebSocket endpoint
        .route("/ws", get(handlers::ws_handler))

        // REST API endpoints
        .route("/api/v1/sessions", get(handlers::list_sessions))
        .route("/api/v1/sessions", post(handlers::create_session))
        .route("/api/v1/sessions/:id", get(handlers::get_session))
        .route("/api/v1/sessions/:id", delete(handlers::delete_session))
        .route("/api/v1/sessions/:id/messages", get(handlers::get_messages))

        // Health check
        .route("/health", get(|| async { "OK" }))

        .layer(CorsLayer::permissive())
        .with_state(session_manager);

    info!("HTTP/WebSocket server listening on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
