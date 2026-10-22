use axum::{
    Router,
    http::HeaderValue,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};

use crate::{config::Config, handlers};

pub fn build_router(config: &Config) -> Router {
    let allowed_origins: Vec<HeaderValue> = config
        .cors_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    Router::new()
        .route("/api/v1/health", get(handlers::health))
        .route("/api/v1/convert", post(handlers::convert))
        .layer(RequestBodyLimitLayer::new(config.max_upload_bytes))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(config.clone())
}
