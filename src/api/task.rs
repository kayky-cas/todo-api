use actix_web::{
    delete, get,
    http::StatusCode,
    post, put,
    web::{Data, Json, Path, ReqData},
    HttpResponse, Responder, Result,
};
use sqlx::{postgres::PgDatabaseError, query_as};
use uuid::Uuid;

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

    let query = query_as!(
        Task,
        "INSERT INTO tasks (name, tag, description, date, user_id) VALUES ($1, $2, $3, $4, $5) returning *",
        body.name,
        body.tag,
        body.description,
        body.date,
        user.id
    )
    .fetch_one(&data.db)
    .await;

    let task = match query {
        Ok(task) => task,
        Err(error) => {
            return Err(error
                .as_database_error()
                .ok_or(ApiError::InternalDatabaseError(None))?
                .downcast_ref::<PgDatabaseError>()
                .into());
        }
    };

    Ok(HttpResponse::build(StatusCode::CREATED).json(task))
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

    let query = match query {
        Ok(query) => query,

        Err(error) => {
            return Err(error
                .as_database_error()
                .ok_or(ApiError::InternalDatabaseError(None))?
                .downcast_ref::<PgDatabaseError>()
                .into())
        }
    };

    let old_task = query.ok_or(ApiError::UnprocessableEntity(Some(
        "Task not found!".to_string(),
    )))?;

    if old_task.user_id != user.id {
        return Err(ApiError::Unauthorized(Some(
            "You don't have persmission to update this task!".to_string(),
        )));
    }

    let query = query_as!(
        Task,
        "UPDATE tasks SET name = $1, description = $2, tag = $3, date = $4 WHERE id = $5 returning *",
        body.name,
        body.description,
        body.tag,
        body.date,
        body.id,
        )
        .fetch_one(&data.db)
        .await;

    let task = match query {
        Ok(task) => task,
        Err(error) => {
            return Err(error
                .as_database_error()
                .ok_or(ApiError::InternalDatabaseError(None))?
                .downcast_ref::<PgDatabaseError>()
                .into());
        }
    };

    Ok(HttpResponse::build(StatusCode::ACCEPTED).json(task))
}

#[delete("/{task_id}")]
pub async fn delete_task(
    user: ReqData<UserId>,
    path: Path<String>,
    data: Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let user = user.into_inner();
    let task_id: Uuid = match path.into_inner().parse() {
        Ok(task_id) => task_id,
        Err(_) => return Err(ApiError::Forbidden(Some("Invalid task id".to_string()))),
    };

    let query = query_as!(
        UserIdTask,
        "SELECT user_id FROM tasks WHERE id = $1",
        task_id
    )
    .fetch_optional(&data.db)
    .await;

    let query = match query {
        Ok(query) => query,

        Err(error) => {
            return Err(error
                .as_database_error()
                .ok_or(ApiError::InternalDatabaseError(None))?
                .downcast_ref::<PgDatabaseError>()
                .into())
        }
    };

    let old_task = query.ok_or(ApiError::UnprocessableEntity(Some(
        "Task not found!".to_string(),
    )))?;

    if old_task.user_id != user.id {
        return Err(ApiError::Unauthorized(Some(
            "You don't have persmission to delete this task!".to_string(),
        )));
    }

    let query = query_as!(Task, "DELETE FROM tasks WHERE id = $1 returning *", task_id)
        .fetch_one(&data.db)
        .await;

    let task = match query {
        Ok(task) => task,
        Err(error) => {
            return Err(error
                .as_database_error()
                .ok_or(ApiError::InternalDatabaseError(None))?
                .downcast_ref::<PgDatabaseError>()
                .into())
        }
    };

    Ok(HttpResponse::build(StatusCode::ACCEPTED).json(task))
}
