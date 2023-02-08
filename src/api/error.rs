use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use derive_more::Display;

#[derive(Debug, Display)]
pub enum ApiError {
    #[display(fmt = "Internal database error!")]
    InternalDatabaseError(Option<String>),
    #[display(fmt = "Internal server error!")]
    InternalServerError(Option<String>),
    #[display(fmt = "Unauthorized request!")]
    Unauthorized(Option<String>),
    #[display(fmt = "Forbiden resource!")]
    Forbidden(Option<String>),
    #[display(fmt = "Bad reuqest!")]
    BadRequest(Option<String>),
    #[display(fmt = "Unprocessable Entity!")]
    UnprocessableEntity(Option<String>),
}

use serde_json::json;
use sqlx::postgres::PgDatabaseError;
use ApiError::*;

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            InternalDatabaseError(_) | InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Forbidden(_) => StatusCode::FORBIDDEN,
            BadRequest(_) => StatusCode::BAD_REQUEST,
            UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let status_code = self.status_code();

        let message: String = match self {
            InternalDatabaseError(Some(message))
            | InternalServerError(Some(message))
            | Unauthorized(Some(message))
            | Forbidden(Some(message))
            | BadRequest(Some(message)) => message.clone(),
            UnprocessableEntity(Some(message)) => message.clone(),
            _ => format!("{}", self),
        };

        HttpResponse::build(status_code)
            .insert_header(ContentType::json())
            .json(json!({
                "status": status_code.as_u16(),
                "message": message
            }))
    }
}

// TODO: find a better place to put this
fn format_detail_from_unique(detail: String) -> String {
    let details: Vec<String> = detail
        .split('(')
        .skip(1)
        .map(|s| s.chars().take_while(|&c| c != ')').collect())
        .collect();

    return format!("The {} {} already exists.", details[0], details[1]);
}

impl From<&PgDatabaseError> for ApiError {
    fn from(value: &PgDatabaseError) -> Self {
        match value.code() {
            "23505" if value.detail().is_some() => {
                let detail = value.detail().unwrap().to_string();
                return BadRequest(Some(format_detail_from_unique(detail)));
            }
            _ => {
                println!("Code not handled: {:?}", value.code());
                return Self::InternalDatabaseError(None);
            }
        }
    }
}
