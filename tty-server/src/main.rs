use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use tracing_subscriber;

mod api;
mod db;
mod protocol;
mod config;
mod session;
mod providers;

use crate::config::Config;
use crate::db::Database;
use crate::session::SessionManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("tty_server=debug,tower_http=debug")
        .init();

    info!("Starting tty-server v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded");

    // Initialize database
    let db = Database::new(&config.server.database_path)?;
    db.migrate()?;
    info!("Database initialized at {}", config.server.database_path);

    // Create shared state
    let session_manager = Arc::new(RwLock::new(SessionManager::new(db.clone(), config.clone())));

    // Spawn native protocol server
    let native_handle = {
        let session_manager = session_manager.clone();
        let bind_addr = config.server.native_bind.clone();
        tokio::spawn(async move {
            if let Err(e) = protocol::server::run(bind_addr, session_manager).await {
                error!("Native protocol server error: {}", e);
            }
        })
    };

    // Spawn HTTP/WebSocket server
    let http_handle = {
        let session_manager = session_manager.clone();
        let bind_addr = config.server.http_bind.clone();
        tokio::spawn(async move {
            if let Err(e) = api::server::run(bind_addr, session_manager, config).await {
                error!("HTTP server error: {}", e);
            }
        })
    };

    info!("Server started successfully");
    info!("  Native protocol: {}", config.server.native_bind);
    info!("  HTTP/WebSocket: {}", config.server.http_bind);

    // Wait for both servers
    tokio::select! {
        _ = native_handle => {
            error!("Native protocol server stopped");
        }
        _ = http_handle => {
            error!("HTTP server stopped");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received");
        }
    }

    info!("Shutting down...");
    Ok(())
}
