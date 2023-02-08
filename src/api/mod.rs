mod auth;
pub mod error;
mod task;

use std::str::FromStr;

use crate::{
    api::auth::Claims,
    model::user::{User, UserId},
};

use self::{
    auth::{login, register},
    error::ApiError,
    task::get_task_by_user,
};
use actix_web::{
    dev::ServiceRequest,
    error::Error,
    web::{self, scope},
    HttpMessage,
};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;

pub fn config(conf: &mut web::ServiceConfig) {
    let bearer_middleware = HttpAuthentication::bearer(jwt_validate);

    let scope = scope("/api")
        .service(scope("/auth").service(login).service(register))
        .service(
            scope("")
                .service(scope("/task").service(get_task_by_user))
                .wrap(bearer_middleware),
        );

    conf.service(scope);
}

async fn jwt_validate(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();

    let secret_key = std::env::var("SECRET_KEY");

    if let Err(_) = secret_key {
        return Err((ApiError::InternalServerError(None).into(), req));
    }

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.unwrap().as_ref()),
        &Validation::default(),
    );

    match token_data {
        Ok(token_data) => {
            let uuid = Uuid::from_str(&token_data.claims.sub);

            if let Err(_) = uuid {
                return Err((
                    ApiError::Unauthorized(Some("Not processable token!".to_string())).into(),
                    req,
                ));
            }

            req.extensions_mut().insert(UserId { id: uuid.unwrap() });
            Ok(req)
        }
        Err(_) => Err((
            ApiError::Unauthorized(Some("Bearer token not valid!".to_string())).into(),
            req,
        )),
    }
}
