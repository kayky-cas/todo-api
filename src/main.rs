mod api;
mod model;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub struct AppState {
    db: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    dotenv().ok();

    let datatbase_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");

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
