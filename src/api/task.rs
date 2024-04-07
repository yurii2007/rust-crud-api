use actix_web::{
    error::ResponseError,
    get,
    http::{header::ContentType, StatusCode},
    post, put,
    web::{Data, Json, Path},
    HttpResponse,
};

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TaskId {
    task_global_id: String,
}

#[get("/tasks/{task_global_id}")]
pub async fn get_task(task_id: Path<TaskId>) -> Json<String> {
    Json(task_id.into_inner().task_global_id)
}
