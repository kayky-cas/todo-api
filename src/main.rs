mod api;
mod model;

use std::env;

use actix_web::{dev::ServiceRequest, error::Error, web, App, HttpServer, Result};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub struct AppState {
    db: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    dotenv().ok();

    let datatbase_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&datatbase_url)
        .await
        .expect("Unable to create the pool connection!");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: pool.clone() }))
            .configure(api::config)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
