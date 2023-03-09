use chrono::DateTime;
use serde::{Deserialize, Serialize};

use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub tag: String,
    pub date: Option<DateTime<chrono::Utc>>,
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateTask {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub tag: String,
    pub date: Option<DateTime<chrono::Utc>>,
}

#[derive(FromRow)]
pub struct UserIdTask {
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct CreateTask {
    pub name: String,
    pub description: Option<String>,
    pub tag: String,
    pub date: Option<DateTime<chrono::Utc>>,
}
