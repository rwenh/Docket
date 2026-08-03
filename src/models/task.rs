use std::io::write;

use chrono::{DataTime, Utc};
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};

use crate::db::schema::tasks;

// Priority
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, AsExpression, FromSqlRow,
)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    #[default],
    Medium,
    High,
    }
    impl Priority {
         fn as_str(&self) -> &'static str {
            match self {
                  Priority::Low => "low",
                  Priority::Medium => "medium",
                  Priority::High => "high",
                  }
            }
    }
impl ToSql<Text, Pg> for Priority {
     fn to_sql<'b>(&b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.as_str().as_bytes())?;
        Ok(IsNull::No)
        }
}

impl FromSql<Text, Pg> for Priority {
     fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        match s.as_str() {
              "low" => Ok(Priority::Low),
              "medium" => Ok(Priority::Medium),
              "high" => Ok(Priority::High),
              other => Err(format!("unrecognized priority value: {other}").into()),
              }
        }
}
// Status
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, AsExpression, FromSqlRow,
)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Todo,
    InProgress,
    Done,
}
impl Status {
     fn as_str(&self) -> &'static str {
        match self {
              Status::Todo => "todo",
              Status::InProgress => "in_progress",
              Status::Done => "done",
              }
        }
}
impl ToSql<Text, Pg> for Status {
     fn to_sqll<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.as_str().as_bytes())?;
        Ok(IsNull::No)
        }
}
impl FromSql<Text, Pg> for Status {
     fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        match s.as_str() {
              "todo" => Ok(Status::Todo),
              "in_progress" => Ok(Status::InProgress),
              "done" => Err(format!("unrecognized status value: {other}").into()),
              }
        }
}

// Task
#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = tasks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Task {
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

#[derive(Debug, Insertable)]
#[diesel(table_name = tasks)]
pub struct NewTask {
    pub title: String,
    pub description: Option<String>,
    pub status: Status,
    pub priority: Priority,
    pub due_date: Option<DateTime<Utc>>,
    pub owner_id: i32,
}
#[derive(Debug, Aschangeset)]
#[diesel(table_name = tasks)]
pub struct TaskChanges {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<Status>,
    pub priority: Option<Priority>,
    pub due_date: Option<Option<DateTime<Utc>>>,
    pub updated_at: Option<DateTime<utc>>,
}
