mod config;
mod conversion;
mod errors;
mod handlers;
mod routes;
mod validation;

#[tokio::main]
async fn main() {
    let app = routes::router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("task running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}