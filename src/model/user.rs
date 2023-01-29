use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::api::error::ApiError;

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl CreateUser {
    pub fn validate(&self) -> Result<Self, String> {
        if self.name.len() < 8 {
            return Err("The name must contains more than 8 letters.".to_string());
        }

        if self.email.len() < 10 {
            return Err("The email must contains more than 10 letters.".to_string());
        }

        if self.password.len() < 8 {
            return Err("The password must contains more than 8 letters.".to_string());
        }

        return Ok(self.clone());
    }
}
