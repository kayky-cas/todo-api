use std::str::FromStr;

use actix_web::{
    get, post,
    web::{Data, Json, ReqData},
    Responder, Result,
};
use sqlx::{postgres::PgDatabaseError, query, query_as};
use uuid::Uuid;

use crate::{
    api::error::ApiError,
    model::{
        task::{CreateTask, Task},
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

    let tasks = query_as!(
        Task,
        "SELECT * FROM tasks WHERE user_id = $1",
        Uuid::from_str(&user.id).unwrap()
    )
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
        Uuid::from_str(&user.id).unwrap()
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
