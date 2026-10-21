use axum::{routing::get, Router};

use crate::handlers::{create_task, get_task, health, list_tasks, root};

pub fn router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/{id}", get(get_task))
}
