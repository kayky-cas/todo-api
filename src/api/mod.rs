mod auth;
pub mod error;
mod task;

use std::str::FromStr;

use crate::{api::auth::Claims, model::user::UserId, AppState};

use self::{
    auth::{delete_user, login, register, update_user},
    error::ApiError,
    task::{create_task, delete_task, get_task_by_user, update_task},
};
use actix_web::{
    dev::ServiceRequest,
    error::Error,
    web::{self, scope, Data},
    HttpMessage,
};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::{postgres::PgDatabaseError, query_as};
use uuid::Uuid;

pub fn config(conf: &mut web::ServiceConfig) {
    let bearer_middleware = HttpAuthentication::bearer(jwt_validate);

    let scope = scope("/api")
        .service(scope("/auth").service(login).service(register))
        .service(
            scope("")
                .service(
                    scope("/task")
                        .service(get_task_by_user)
                        .service(create_task)
                        .service(delete_task)
                        .service(update_task),
                )
                .service(scope("/user").service(update_user).service(delete_user))
                .wrap(bearer_middleware),
        );

    conf.service(scope);
}

async fn jwt_validate(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    let data = req.app_data::<web::Data<AppState>>().unwrap();

    let secret_key = match std::env::var("SECRET_KEY") {
        Ok(secret_key) => secret_key,
        Err(_) => return Err((ApiError::InternalServerError(None).into(), req)),
    };

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(),
    );

    match token_data {
        Ok(token_data) => {
            let user_id = match Uuid::from_str(&token_data.claims.sub) {
                Ok(user_id) => user_id,
                Err(_) => {
                    return Err((
                        ApiError::Unauthorized(Some("Not processable token!".to_string())).into(),
                        req,
                    ))
                }
            };

            let query = query_as!(UserId, "SELECT id FROM users WHERE id = $1", user_id)
                .fetch_optional(&data.db)
                .await;

            let user = match query {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err((
                        ApiError::Unauthorized(Some("User doesn't exists anymore!".to_string()))
                            .into(),
                        req,
                    ))
                }
                Err(_) => return Err((ApiError::InternalDatabaseError(None).into(), req)),
            };

            req.extensions_mut().insert(UserId { id: user.id });
            Ok(req)
        }
        Err(_) => Err((
            ApiError::Unauthorized(Some("Bearer token not valid!".to_string())).into(),
            req,
        )),
    }
}
