mod auth;
pub mod error;

use self::auth::{login, register};
use actix_web::web::{self, scope};

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = scope("/api").service(scope("/auth").service(login).service(register));

    conf.service(scope);
}
