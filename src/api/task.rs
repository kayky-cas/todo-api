use actix_web::{
    get, post, put,
    web::{Data, Json, ReqData},
    HttpResponse, Responder, Result,
};
use sqlx::{postgres::PgDatabaseError, query, query_as};

use crate::{
    api::error::ApiError,
    model::{
        task::{CreateTask, Task, UserIdTask},
        user::UserId,
    },
    AppState,
};

#[get("")]
pub async fn get_task_by_user(
    user: ReqData<UserId>,
    data: Data<AppState>,
) -> Result<Json<Vec<Task>>, ApiError> {
    let user = user.into_inner();

    let tasks = query_as!(Task, "SELECT * FROM tasks WHERE user_id = $1", user.id)
        .fetch_all(&data.db)
        .await;

    if let Err(error) = tasks {
        return Err(error
            .as_database_error()
            .ok_or(ApiError::InternalDatabaseError(None))?
            .downcast_ref::<PgDatabaseError>()
            .into());
    }

    Ok(Json(tasks.unwrap()))
}

#[post("")]
pub async fn create_task(
    body: Json<CreateTask>,
    data: Data<AppState>,
    user: ReqData<UserId>,
) -> Result<impl Responder, ApiError> {
    let user = user.into_inner();

    let query = query!(
        "INSERT INTO tasks (name, tag, user_id) VALUES ($1, 'Easy', $2)",
        body.name,
        user.id
    )
    .execute(&data.db)
    .await;

    if let Err(error) = query {
        return Err(error
            .as_database_error()
            .ok_or(ApiError::InternalDatabaseError(None))?
            .downcast_ref::<PgDatabaseError>()
            .into());
    }

    Ok("")
}

#[put("")]
async fn update_task(
    body: Json<Task>,
    data: Data<AppState>,
    user: ReqData<UserId>,
) -> Result<impl Responder, ApiError> {
    let user = user.into_inner();

    let query = query_as!(
        UserIdTask,
        "SELECT user_id FROM tasks WHERE id = $1",
        body.id
    )
    .fetch_optional(&data.db)
    .await;

    if let Err(error) = query {
        return Err(error
            .as_database_error()
            .ok_or(ApiError::InternalDatabaseError(None))?
            .downcast_ref::<PgDatabaseError>()
            .into());
    }

    let old_task = query.unwrap().ok_or(ApiError::UnprocessableEntity(Some(
        "Task not found!".to_string(),
    )))?;

    if old_task.user_id != user.id {
        return Err(ApiError::Unauthorized(Some(
            "You don't have persmission to update this task!".to_string(),
        ))
        .into());
    }

    let query = query_as!(
        Task,
        "UPDATE tasks SET name = $1, description = $2, tag = $3, date = $4 WHERE id = $5",
        body.name,
        body.description,
        body.tag,
        body.date,
        body.id
    )
    .execute(&data.db)
    .await;

    if let Err(error) = query {
        return Err(error
            .as_database_error()
            .ok_or(ApiError::InternalDatabaseError(None))?
            .downcast_ref::<PgDatabaseError>()
            .into());
    }

    return Ok(HttpResponse::Ok());
}
