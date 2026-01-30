//! Static file serving with embedded React frontend

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../frontend/dist"]
struct Assets;

/// Handle static file requests
pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    
    // Try to serve the exact path
    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(Body::from(content.data.into_owned()))
            .unwrap();
    }
    
    // For SPA routing, serve index.html for non-file paths
    if !path.contains('.') || path.is_empty() {
        if let Some(content) = Assets::get("index.html") {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html")
                .body(Body::from(content.data.into_owned()))
                .unwrap();
        }
    }
    
    // Return 404 for missing files
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap()
}
