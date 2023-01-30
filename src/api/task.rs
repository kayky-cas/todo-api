use actix_web::{
    get,
    web::{Data, Json},
    Responder, Result,
};
use sqlx::query_as;

use crate::{api::error::ApiError, model::task::Task, AppState};

#[get("")]
pub async fn get_task_by_user(data: Data<AppState>) -> Result<impl Responder> {
    let tasks = query_as!(Task, "SELECT * FROM tasks")
        .fetch_all(&data.db)
        .await;

    if let Err(query) = tasks {
        return Err(ApiError::from(query.as_database_error().unwrap().downcast_ref()).into());
    }

    Ok(Json(tasks.unwrap()))
}
