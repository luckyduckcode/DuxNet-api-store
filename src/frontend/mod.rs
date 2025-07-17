//! # Frontend Module
//!
//! This module provides the integration layer for serving the web frontend of the DuxNet platform.
//! It serves static web assets and the main index.html for the Tauri desktop app and web interface.
//!
//! ## Purpose
//! - Exposes Axum routes for serving static files and the main HTML entry point.
//! - Bridges the Rust backend with the web-based frontend (Tauri or browser).
//!
//! ## Main Components
//! - `create_frontend_router`: Returns a router for serving static assets and index.html.
//! - `serve_index`: Handler for serving the main HTML file.
//! - `serve_static_file`: Handler for serving static assets from the `static/` directory.
//!
//! ## Best Practices
//! - Keep frontend serving logic minimal and focused on static asset delivery.
//! - Use clear route definitions for static and index serving.
//! - If frontend logic grows, consider splitting into submodules (e.g., for API proxying, SSR, etc.).
//!
//! ## Future Improvements
//! - Add support for frontend asset versioning or cache busting.
//! - Integrate with frontend build tools for automated asset management.
//! - Expand to support additional frontend frameworks or SSR if needed.
//!
//! This structure ensures the frontend integration is simple, reliable, and easy to extend as the platform evolves.
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, Response},
    routing::get,
    Router,
};
use std::fs;
use tower_http::services::ServeDir;

pub fn create_frontend_router() -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .nest_service("/static", ServeDir::new("static"))
}

async fn serve_index() -> Result<Html<String>, StatusCode> {
    let html_content = include_str!("../../static/index.html");
    Ok(Html(html_content.to_string()))
}

pub async fn serve_static_file(Path(path): Path<String>) -> Result<Response<String>, StatusCode> {
    let file_path = format!("static/{}", path);
    
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(content)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(response)
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
} 