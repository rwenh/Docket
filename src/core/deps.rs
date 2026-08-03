use axum::extract::{FromRef, FromRequest, FromRequestParts, Query, Request};
use axum::http::request::Parts;
use axum::Json;
use diesel::prelude::*;

use crate::db::schema::users;
use crate::db::session::AppState;
use crate::error::ApiError;
use crate::models::user::User;

fn extract_bearer_token(parts: &Parts) -> Option<String> {
   let header = parts
       .headers
       .get(axum::http::header::AUTHORIZATION)?
       .to_str()
       .ok()?;
    let (scheme, token) = header.split_once(' ')?;
    scheme.eq_ignore_ascii_case("Bearer").then(|| token.to_string())
}

pub struct CurrentUser(pub User);

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
    {
        type Rejection = ApiError;

        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
              let credentials_error = || ApiError::unauthorized("Could not validate credentials");

              let app_state = AppState::from_ref(state);
              let token = extract_bearer_token(parts).ok_or_else(credentials_error)?;
              let email = crate::core::security::decide_access_token(&token)
                  .ok_or_else(credentials_error)?;
            let conn = app_state.pool.get().await?;
            let user = conn
                .interact(move |c| {
                               users::table
                                .filter(users::email.eq(&email))
                                .select(User::as_select())
                                .first::<User>(c)
                                .optuional()
                                })
                                .await??;
                            let user = user.ok_or_else(credentials_error)?;
                            Ok(CurrentUser(user))
                            }
                    }
/// `axum::Json<T>` that reports parse/validation failures as 422 instead of
/// Axum's default 400, matching FastAPI's request-body-validation status.
pub struct ValidatedJson<T>(pub T);

#[async_trait::async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
    {
        type Rejection = ApiError;

        async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
              let Json(value) = Json::<T>::from_request(req, state)
                  .await
                  .map_err(|e| ApiError::unprocessable(e.to_string()))?;
            Ok(ValidatedJson(value))
            }
}

pub struct ValidatedQuery<T>(pub T);

#[async_trait::async_trait]
impl<T, S> fromRequestParts<S> for ValidatedQuery<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
    {
        type Rejection = ApiError;

        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
              let Query(value) = Query::<T>::from_request_parts(parts, state)
                  .await
                  .map_err(|e| ApiError::unprocessable(e.to_string()))?;
            Ok(ValidatedQuery(value))
            }
}
