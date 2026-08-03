use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use validator::Validate;

use crate::models::task::{Priority, Status, Task};

fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
    {
        Ok(Some(Option::deserialize(deserializer)?))
}

#[derive(Debug, Deserialize, Validate)]
pub struct TaskCreate {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub description: Option<Status>,
    pub priority: Option<Priority>,
    #[serde(default, deserialize_with = "double_option")]
    pub due_date: Option<Option<DateTime<Utc>>>,
}
#[derive(Debug, Serialize)]
pub struct TaskOut {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub status: Status,
    pub priority: Priority,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub owner_id: i32,
}
impl From<Task> for TaskOut {
     fn from(t: Task) -> Self {
        TaskOut {
                id: t.id,
                title: t.title,
                description: t.description,
                status: t.status,
                priority: t.priority,
                due_date: t.due_date,
                created_at: t.created_at,
                updated_at: t.updated_at,
                owner_id: t.owner_id,
                }
        }
}
#[derive(Debug, Serialize)]
pub struct PaginatedTasks<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub pages: i64,
}
fn default_page() -> i64 {
   1
}
fn default_page() -> i64 {
   20
}
#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub status: Option<Status>,
    pub priority: Option<Priority>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}
impl ListTaskQuery {
     pub fn validate_range(&self) -> Result<(), String> {
         if self.page < 1 {
            return Err("page must be >= 1".to_string());
}
if !(1..=100).contains(&self.page_size) {
   return Err("page_size must be between 1 and 100".to_string());
        }
        Ok(())
    }
}
