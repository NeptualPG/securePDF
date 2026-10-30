use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::HeaderValue,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{config::Config, handlers};

const MULTIPART_OVERHEAD_BYTES: usize = 64 * 1024;

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
        // Applied through the extractor so oversized uploads get the JSON 413 error.
        // The margin leaves room for the multipart boundaries and text fields;
        // the handler enforces the exact file size.
        .layer(DefaultBodyLimit::max(
            config
                .max_upload_bytes
                .saturating_add(MULTIPART_OVERHEAD_BYTES),
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(config.clone())
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    const BOUNDARY: &str = "securepdf-test-boundary";

    fn test_config() -> Config {
        let temp_dir = std::env::temp_dir().join("securepdf-tests");
        std::fs::create_dir_all(&temp_dir).expect("create temp dir");
        Config {
            max_upload_bytes: 1024,
            temp_dir: temp_dir.to_string_lossy().into_owned(),
            ..Config::from_env()
        }
    }

    fn convert_request(file: &[u8], file_name: &str) -> Request<Body> {
        let mut body = format!(
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"target_format\"\r\n\r\npdf\r\n\
             --{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\r\n"
        )
        .into_bytes();
        body.extend_from_slice(file);
        body.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());

        Request::post("/api/v1/convert")
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={BOUNDARY}"),
            )
            .body(Body::from(body))
            .expect("build request")
    }

    async fn send(request: Request<Body>) -> (StatusCode, String) {
        let response = build_router(&test_config())
            .oneshot(request)
            .await
            .expect("router response");
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("read body")
            .to_bytes();
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let request = Request::get("/api/v1/health")
            .body(Body::empty())
            .expect("build request");
        assert_eq!(send(request).await, (StatusCode::OK, "ok".to_string()));
    }

    #[tokio::test]
    async fn oversized_upload_returns_json_413() {
        let (status, body) = send(convert_request(&[b'a'; 4096], "big.pdf")).await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert!(body.contains(r#""code":"PAYLOAD_TOO_LARGE""#), "{body}");
    }

    #[tokio::test]
    async fn upload_over_the_body_limit_returns_json_413() {
        let file = vec![b'a'; 1024 + MULTIPART_OVERHEAD_BYTES];
        let (status, body) = send(convert_request(&file, "big.pdf")).await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert!(body.contains(r#""code":"PAYLOAD_TOO_LARGE""#), "{body}");
    }

    #[tokio::test]
    async fn renamed_file_returns_json_400() {
        let (status, body) = send(convert_request(b"not a pdf", "fake.pdf")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains(r#""code":"CORRUPT_FILE""#), "{body}");
    }
}
