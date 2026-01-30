//! Web server module

mod routes;
mod static_files;
mod websocket;

pub use routes::resume_incomplete_downloads;

use crate::AppState;
use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

/// Run the web server
pub async fn run(state: Arc<AppState>) -> Result<()> {
    let settings = state.settings.read().clone();
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    
    // Resume any incomplete downloads from previous session
    resume_incomplete_downloads(state.clone());
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    let app = Router::new()
        .nest("/api", routes::api_routes())
        .route("/ws", axum::routing::get(websocket::ws_handler))
        .fallback(static_files::static_handler)
        .layer(cors)
        .with_state(state);
    
    info!("Starting web server on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
