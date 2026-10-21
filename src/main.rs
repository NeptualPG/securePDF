mod config;
mod conversion;
mod errors;
mod handlers;
mod routes;
mod validation;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info,axum=debug")
        .init();
    let config = config::Config::from_env();
    std::fs::create_dir_all(&config.temp_dir).expect("failed to create temporary directory");
    let address = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .unwrap();

    tracing::info!(%address, "SecurePDF backend listening");

    axum::serve(listener, routes::build_router(&config))
        .await
        .unwrap();
}