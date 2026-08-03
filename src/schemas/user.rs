use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::user::user;

#[derive(Debug, Deserialize, Validate)]
pub struct UserCreate {
    #[validate(email)]
    pub email: String,
    pub password: String,
}
#[derive(Debug, Serialize)]
pub struct UserOut {
    pub id: i32,
    pub email: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
impl From<User> for UserOut {
     fn from(u: User) -> Self {
        UserOut {
            id: u.id,
            email: u.email,
            is_active: u.is_active,
            created_at: u.created_at,
            }
    }
}
#[derive(Debug, serialize)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
}

impl Token {
     pub fn new(acess_token: String) -> Self {
     Self { access_token, token_type: "bearer".to_string() }
     }
}
