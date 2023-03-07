use actix_web::{
    put,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use sqlx::{postgres::PgDatabaseError, query};

use crate::{
    api::error::ApiError,
    model::user::{CreateUser, UserId},
    AppState,
};

#[put("")]
async fn update_user(
    body: Json<CreateUser>,
    data: Data<AppState>,
    user: ReqData<UserId>,
) -> Result<impl actix_web::Responder, ApiError> {
    let user = user.into_inner();

    let query = query!(
        "UPDATE users SET name = $1, email = $2, password = $3, image = $4 WHERE id = $5",
        body.name,
        body.email,
        body.password,
        body.image,
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

    return Ok(HttpResponse::Ok());
}
