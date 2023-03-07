use actix_web::{get, Responder};

#[get("")]
pub async fn health() -> impl Responder {
    "🤓"
}
