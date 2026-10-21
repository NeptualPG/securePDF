use std::collections::HashMap;

use axum::extract::{Path, Query};

pub async fn root() -> &'static str {
    "taskr is alive"
}

pub async fn health() -> &'static str {
    "Ok"
}

pub async fn list_tasks(Query(params): Query<HashMap<String, String>>) -> String {
    let filter = params
        .get("done")
        .map_or("everything".to_string(), |value| format!("done={value}"));

    format!("Listing tasks, filter: {filter}")
}

pub async fn create_task() -> &'static str {
    "a task will be created here"
}

pub async fn get_task(Path(id): Path<u32>) -> String {
    format!("you asked for task {id}")
}
