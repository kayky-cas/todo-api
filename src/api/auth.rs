use actix_web::{
    post,
    web::{Data, Json},
    Responder, Result,
};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgDatabaseError, query, query_as};

use crate::{
    api::error::ApiError,
    model::user::{CreateUser, LoginUser, User, UserInfo},
    AppState,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[post("")]
pub async fn login(
    body: Json<LoginUser>,
    data: Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let user = query_as!(User, "SELECT * FROM users WHERE email = $1", body.email)
        .fetch_optional(&data.db)
        .await
        .map_err(|_| ApiError::InternalDatabaseError(None))?;

    if user.is_none() {
        return Err(ApiError::Forbidden(Some(
            "Incorrect email or password!".to_string(),
        )));
    }

    let user = user.unwrap();

    let secret_key =
        std::env::var("SECRET_KEY").map_err(|_| ApiError::InternalServerError(None))?;

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(60))
        .ok_or(ApiError::InternalServerError(None))?
        .timestamp();

    let claim = Claims {
        sub: user.id.to_string(),
        exp: expiration as usize,
    };

    let token = encode(
        &jsonwebtoken::Header::default(),
        &claim,
        &EncodingKey::from_secret(&secret_key.as_ref()),
    )
    .map_err(|_| ApiError::InternalServerError(None))?;

    let user = UserInfo {
        name: user.name,
        email: user.email,
    };

    Ok(Json(json!({ "user": user, "token": token })))
}

// TODO: create a good response
#[post("/register")]
pub async fn register(
    body: Json<CreateUser>,
    data: Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let user = body
        .validate()
        .map_err(|message| ApiError::BadRequest(Some(message)))?;

    let query = query!(
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3)",
        user.name,
        user.email,
        user.password
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

    return Ok("");
}
