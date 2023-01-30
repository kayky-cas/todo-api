use actix_web::{
    http::header::Encoding,
    post,
    web::{Data, Json},
    Responder, Result,
};
use jsonwebtoken::{encode, EncodingKey};
use serde_json::json;
use sqlx::{postgres::PgDatabaseError, query, query_as};

use crate::{
    api::error::ApiError,
    model::user::{CreateUser, LoginUser, User, UserInfo},
    AppState,
};

#[post("")]
pub async fn login(body: Json<LoginUser>, data: Data<AppState>) -> Result<impl Responder> {
    let user = query_as!(User, "SELECT * FROM users WHERE email = $1", body.email)
        .fetch_optional(&data.db)
        .await
        .map_err(|_| ApiError::InternalDatabaseError(None))?;

    if user.is_none() {
        return Err(ApiError::Forbidden(Some("Incorrect email or password!".to_string())).into());
    }

    let user = user.unwrap();

    let secret_key =
        std::env::var("SECRET_KEY").map_err(|_| ApiError::InternalServerError(None))?;

    let token = encode(
        &jsonwebtoken::Header::default(),
        &user,
        &EncodingKey::from_secret(&secret_key.as_ref()),
    )
    .map_err(|_| ApiError::InternalServerError(None))?;

    let user = UserInfo {
        name: user.name,
        email: user.email,
    };

    Ok(Json(json!({ "user": user, "token": token })))
}

#[post("/register")]
pub async fn register(body: Json<CreateUser>, data: Data<AppState>) -> Result<impl Responder> {
    let body = body.validate();

    if let Err(message) = body {
        return Err(ApiError::BadRequest(Some(message)).into());
    }

    let body = body.unwrap();

    let query = query!(
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3)",
        body.name,
        body.email,
        body.password
    )
    .execute(&data.db)
    .await;

    if let Err(error) = query {
        let error: &PgDatabaseError = error.as_database_error().unwrap().downcast_ref();

        return Err(ApiError::from(error).into());
    }

    return Ok("");
}
