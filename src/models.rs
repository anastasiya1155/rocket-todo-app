use serde::{Deserialize, Serialize};
use diesel::{Queryable, Insertable};
use chrono::{DateTime, Utc};

use crate::schema::todos;

#[derive(Serialize, Deserialize, Queryable, Debug, Insertable)]
#[table_name = "todos"]
pub struct Todo<> {
    pub id: i32,
    pub title: String,
    pub done: bool,
    pub created_at: DateTime<Utc>,
}
