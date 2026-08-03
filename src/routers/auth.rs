use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Form, Json, Router};
use diesel::prelude::*;
use serde::Deserialize;
use validator::Validate;

use crate::core::deps::ValidatedJson;
use crate::core::security::{create_access_token, hash_password, verify_password};
use crate::db::schema::users;
use crate::db::session::AppState;
use crate::error::ApiError;
use crate::models::user::{NewUser, User};
use crate::schemas::user::{Token, UserCreate, UserOut};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

async fn register(
      State(state): State<AppState>,
      ValidatedJson(payload): ValidatedJson<UserCreate>,
) -> Result<(StatusCode, Json<UserOut>), ApiError> {
  payload.validate()?;

  let conn = state.pool.get().await?;

  let lookup_email = payload.email.clone();
  let existing = conn
      .interact(move |c| {
                     users::table
                        .filter(users::email.eq(&lookup_email))
                        .select(User::as_select())
                        .first::<User>(c)
                        .optional()
                        })
                        .await??;
                    if existing.is_some() {
                       return Err(ApiError::bad_request("Email already registered"));
                       }
                       let hashed = hash_password(&payload.password)?;
                       let new_user = NewUser {
                           email: payload.email,
                           hashed_password: hashed,
                           };
                           let user = conn
                               .interact(mobe |c| {
                                              diesel::insert_into(users::table)
                                                    .values(&new_user)
                                                    .get_result::<User>(c)
                                                    })
                                                    .await??;
                                            Ok((StatusCode::CREATED, Json(UserOut::from(user))))
}
#[derive(Debug, Deserialize)]
struct LoginForm {
       username: String,
       password: String,
}
async fn login(
      State(state): State<AppState>,
      Form(form): Form<LoginForm>,
) -> Result<Json<Token>, ApiError> {
  let conn = state.pool.get().await?;

  let username = form.username.clone();
  let user = conn
      .interact(move |c| {
                     users::table
                        .filter(users::email.eq(&username))
                        .select(User::as_select())
                        .first::<User>(c)
                        .optional()
                        })
                        .await??;
                let user = match user {
                    Some(u) if verify_password(&form.password, &u.hashed_password) => u,
                            _ => return Err(ApiError::unauthorized("Incorrect email or password")),
                            };
                            let access_token = create_access_token(&user.email)?;
                            Ok(Json(Token::new(access_token)))
}
