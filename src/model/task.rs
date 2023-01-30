use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub tag: String,
    pub date: Option<NaiveDate>,
}
