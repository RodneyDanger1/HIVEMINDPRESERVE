//! Embedded web UI assets (`web/dist`, built by the frontend).
//!
//! The single Rust binary serves the built SPA from memory via `rust-embed`:
//! `GET /` and every non-API path return `index.html` (SPA routing), real asset
//! paths return the embedded bytes with the correct content type, and unknown
//! asset paths return an honest 404. Lookup is by exact archive path, so a
//! path-traversal request can never escape `web/dist`.

use axum::http::{header, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

/// Embedded `web/dist` tree (compile-time embed; rebuild to pick up UI changes).
#[derive(RustEmbed)]
#[folder = "../../../web/dist"]
struct WebAssets;

/// Content type for an embedded asset path.
#[must_use]
pub fn content_type(path: &str) -> &'static str {
    let extension = path.rsplit('.').next().unwrap_or_default().to_ascii_lowercase();
    match extension.as_str() {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

/// Router fallback: SPA + embedded assets, 404 for unknown asset/API paths.
pub async fn static_ui(method: Method, uri: Uri) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return (StatusCode::METHOD_NOT_ALLOWED, "method not allowed").into_response();
    }
    let path = uri.path().trim_start_matches('/');
    // API/transport paths that fell through to the fallback are real 404s.
    if path.starts_with("api/") || path == "health" || path == "ws" {
        return json_not_found();
    }

    let requested = if path.is_empty() { "index.html" } else { path };
    if let Some(asset) = WebAssets::get(requested) {
        return (
            [(header::CONTENT_TYPE, content_type(requested))],
            asset.data.into_owned(),
        )
            .into_response();
    }

    // SPA route (no file extension) -> index.html; the client router renders it.
    if !requested.contains('.') {
        if let Some(index) = WebAssets::get("index.html") {
            return (
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                index.data.into_owned(),
            )
                .into_response();
        }
    }
    json_not_found()
}

fn json_not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        axum::Json(serde_json::json!({
            "error": "not found",
            "schema": hiverouter_contract::SCHEMA_VERSION,
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_types_are_stable() {
        assert_eq!(content_type("index.html"), "text/html; charset=utf-8");
        assert_eq!(content_type("assets/index-abc.js"), "text/javascript; charset=utf-8");
        assert_eq!(content_type("assets/index-abc.css"), "text/css; charset=utf-8");
        assert_eq!(content_type("favicon.ico"), "image/x-icon");
        assert_eq!(content_type("unknown.bin"), "application/octet-stream");
    }

    #[test]
    fn embedded_index_exists_and_contains_the_app_root() {
        let index = WebAssets::get("index.html").expect("web/dist/index.html embedded");
        let text = String::from_utf8_lossy(&index.data);
        assert!(text.contains("<div id=\"root\">"), "index.html missing app root");
    }
}
