use crate::{
    model::task::{Task, TaskState},
    repository::ddb::DDBRepository,
};

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

#[derive(Deserialize)]
pub struct SubmitTaskRequest {
    user_id: String,
    task_type: String,
    source_file: String,
}

#[derive(Debug, Display)]
pub enum TaskError {
    TaskNotFound,
    TaskUpdateFailure,
    TaskCreationFailure,
    BadTaskRequest,
}

impl ResponseError for TaskError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            TaskError::BadTaskRequest => StatusCode::BAD_REQUEST,
            TaskError::TaskCreationFailure => StatusCode::FAILED_DEPENDENCY,
            TaskError::TaskNotFound => StatusCode::NOT_FOUND,
            TaskError::TaskUpdateFailure => StatusCode::FAILED_DEPENDENCY,
        }
    }
}

#[get("/tasks/{task_global_id}")]
pub async fn get_task(
    ddb_repo: Data<DDBRepository>,
    task_id: Path<TaskId>,
) -> Result<Json<Task>, TaskError> {
    let task = ddb_repo.get_task(task_id.into_inner().task_global_id).await;

    match task {
        Some(task) => Ok(Json(task)),
        None => Err(TaskError::TaskNotFound),
    }
}

#[put("/tasks/{task_global_id}/start")]
pub async fn start_task(
    ddb_repo: Data<DDBRepository>,
    task_id: Path<TaskId>,
) -> Result<Json<TaskId>, TaskError> {
    state_transition(
        ddb_repo,
        task_id.into_inner().task_global_id,
        TaskState::InProgress,
        None,
    )
    .await
}

#[post("/tasks")]
pub async fn submit_task(
    ddb_repo: Data<DDBRepository>,
    request: Json<SubmitTaskRequest>,
) -> Result<Json<TaskId>, TaskError> {
    let task = Task::new(
        request.user_id.clone(),
        request.task_type.clone(),
        request.source_file.clone(),
    );

    let task_id = task.get_global_id();
    match ddb_repo.put_task(task).await {
        Ok(()) => Ok(Json(TaskId {
            task_global_id: task_id,
        })),
        Err(_) => Err(TaskError::TaskCreationFailure),
    }
}

async fn state_transition(
    ddb_repo: Data<DDBRepository>,
    task_global_id: String,
    new_state: TaskState,
    result_file: Option<String>,
) -> Result<Json<TaskId>, TaskError> {
    let mut task = match ddb_repo.get_task(task_global_id).await {
        Some(task) => task,
        None => return Err(TaskError::TaskNotFound),
    };

    if !task.can_transition_to(&new_state) {
        return Err(TaskError::BadTaskRequest);
    };

    task.state = new_state;
    task.result_file = result_file;

    let task_id = task.get_global_id();
    match ddb_repo.put_task(task).await {
        Ok(()) => Ok(Json(TaskId {
            task_global_id: task_id,
        })),
        Err(_) => Err(TaskError::TaskUpdateFailure),
    }
}
