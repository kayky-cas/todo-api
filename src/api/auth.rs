use actix_web::{
    delete,
    http::StatusCode,
    post, put,
    web::{Data, Json, ReqData},
    HttpResponse, Responder, Result,
};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgDatabaseError, query_as};

use crate::{
    api::error::ApiError,
    model::user::{CreateUser, LoginUser, User, UserId, UserInfo},
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
        .checked_add_signed(chrono::Duration::minutes(120))
        .ok_or(ApiError::InternalServerError(None))?
        .timestamp();

    let claim = Claims {
        sub: user.id.to_string(),
        exp: expiration as usize,
    };

    let token = encode(
        &jsonwebtoken::Header::default(),
        &claim,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .map_err(|_| ApiError::InternalServerError(None))?;

    let user = UserInfo {
        name: user.name,
        email: user.email,
        image: user.image,
    };

    Ok(Json(json!({ "user": user, "token": token })))
}

type ResponseUser = UserInfo;

#[post("/register")]
pub async fn register(
    body: Json<CreateUser>,
    data: Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let user = body
        .validate()
        .map_err(|message| ApiError::UnprocessableEntity(Some(message)))?;

    let query = query_as!(
        ResponseUser,
        "INSERT INTO users (name, email, password, image) VALUES ($1, $2, $3, $4) returning name, email, image",
        user.name,
        user.email,
        user.password,
        user.image
    )
    .fetch_one(&data.db)
    .await;

    let user = match query {
        Ok(user) => user,
        Err(error) => {
            return Err(error
                .as_database_error()
                .ok_or(ApiError::InternalDatabaseError(None))?
                .downcast_ref::<PgDatabaseError>()
                .into())
        }
    };

    Ok(HttpResponse::build(StatusCode::CREATED).json(user))
}

#[put("")]
async fn update_user(
    body: Json<CreateUser>,
    data: Data<AppState>,
    user: ReqData<UserId>,
) -> Result<impl Responder, ApiError> {
    let user = user.into_inner();

    let query = query_as!(
        ResponseUser,
        "UPDATE users SET name = $1, email = $2, password = $3, image = $4 WHERE id = $5 returning name, email, image",
        body.name,
        body.email,
        body.password,
        body.image,
        user.id
    )
    .fetch_one(&data.db)
    .await;

    let user = match query {
        Ok(user) => user,
        Err(error) => {
            return Err(error
                .as_database_error()
                .ok_or(ApiError::InternalDatabaseError(None))?
                .downcast_ref::<PgDatabaseError>()
                .into());
        }
    };

    Ok(HttpResponse::build(StatusCode::ACCEPTED).json(user))
}

#[delete("")]
pub async fn delete_user(
    user: ReqData<UserId>,
    data: Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let user = user.into_inner();

    let query = query_as!(
        ResponseUser,
        "DELETE FROM users WHERE id = $1 returning name, email, image",
        user.id
    )
    .fetch_one(&data.db)
    .await;

    let user = match query {
        Ok(user) => user,
        Err(error) => {
            return Err(error
                .as_database_error()
                .ok_or(ApiError::InternalDatabaseError(None))?
                .downcast_ref::<PgDatabaseError>()
                .into())
        }
    };

    Ok(HttpResponse::build(StatusCode::ACCEPTED).json(user))
}
