//! WebSocket handler for real-time progress updates

use crate::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tracing::{error, info};

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to progress updates
    let mut progress_rx = state.download_manager.subscribe();
    
    info!("WebSocket client connected");
    
    // Spawn task to forward progress updates to client
    let send_task = tokio::spawn(async move {
        while let Ok(update) = progress_rx.recv().await {
            let msg = serde_json::to_string(&update).unwrap_or_default();
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });
    
    // Handle incoming messages (for future bidirectional communication)
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Handle client messages if needed
                info!("Received WebSocket message: {}", text);
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket client disconnected");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
    
    // Cancel the send task when client disconnects
    send_task.abort();
}
