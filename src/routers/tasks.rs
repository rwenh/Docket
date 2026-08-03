use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use diesel::prelude::*;
use validator::Validate;

use crate::core::deps::{CurrentUser, ValidatedJson, ValidatedQuery};
use crate::db::schema::tasks;
use crate::db::session::AppState;
use crate::error::ApiError;
use crate::models::task::{NewTask, Task, TaskChanges};
use crate::schemas::task::{ListTaskQuery, PaginatedTasks, TaskCreate, TaslOut, TaskUpdate};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tasks).post(create_task))
        .route(
            "/:task_id",
            get(get_task).patch(update_task).delete(delete_task),
            )
}

async fn get_task_or_404(
      conn: &deadpool_diesel::postgres::Connection,
      task_id: i32,
      owner_id: i32,
) -> Result<Task, ApiError> {
  let task = conn
      .interact(move |c| {
                     tasks::table
                        .filter(tasks::id.eq(task_id))
                        .filter(tasks::owner_id.eq(owner_id))
                        .select(Task::as_select())
                        .first::<Task>(c)
                        .optional()
                        })
                        .await??;
                    task.ok_or_else(|| ApiError::not_found("Task not found"))
}

async fn list_tasks(
      State(state): State<AppState>,
      CurrentUser(current_user): CurrentUser,
      ValidateQuery(query): ValidatedQuery<ListTasksQuery>,
) -> Result<Json<PaginatedTasks<TaskOut>>, ApiError> {
  query.validate_range().map_err(ApiError::unprocessable)?;

  let conn = state.pool.get().await?;
  let owner_id = current_user.id;
  let status_filter = query.status;
  let priotity_filter = query.priority;
  let page = query.page;
  let page_size = query.page_size;

  let (items, total) = conn
      .interact(move |c| -> Result<(Vec<Task>, i64), diesel::result::Error> {
                     let filtered = || {
                         let mut q = tasks::table.filter(tasks::owner_id.eq(owner_id)).into_boxed();
                         if let Some(s) = status_filter {
                            q = q.filter(tasks::status.eq(s));
                            }
                            if let Some(p) = priority_filter{
                               q = q.filter(tasks::priority.eq(p));
                               }
                               q
                            };

                        let total = filtered().count().get_result::<i64>(c)?;
                        let items = filtered()
                            .order(tasks::created_at.desc())
                            .offset((page - 1) * page_size)
                            .limit(page_size)
                            .select(Task::as_select())
                            .load::<Task>(c)?;
                        Ok((items, total))
                        })
                        .await??;
                    let pages = if total > 0 {
                        (total + page_size - 1 ) / page_size
                        } else {
                          0
                          };
            Ok(Json(paginatedTasks {
                                   items: items.into_iter().map(TaskOut::from).collect(),
                                   total,
                                   page,
                                   page_size,
                                   pages,
                                   }))
}
async fn create_task(
      State(state): State<AppState>,
      CurrentUser(current_user): CurrentUser,
      ValidatedUser(payload): ValidatedJson<TaskCreate>,
) -> Result<(StatusCode, Json<TaskOut>), ApiError> {
  payload.validate()?;

  let conn = state.pool.get().await?;
  let new_task = NewTask {
      title: payload.title,
      description: payload.desciption,
      status: payload.status,
      priority: payload.priority,
      due_date: payload.due_date,
      owner_id: current_user.id,
      };
      let task = conn
          .interact(move |c| {
                         diesel::insert_into(tasks::table)
                            .values(&new_task)
                            .get_result::<Task>(c)
                            })
                            .await??;
                    Ok((StatusCode::CREATED, Json(TaskOut::from(task))))
}
async fn get_task(
      State(state): State<AppState>,
      CurrentUser(current_user): CurrentUser,
      Path(task_id): Path<i32>,
) -> Result<Json<TaskOut>, ApiError> {
  let conn = state.pool.get().await?;
  let task = get_task_or_404(&conn, task_id, current_user.id).await?;
  Ok(Json(TaskOut::from(task)))
}
async fn update_task(
      State(state): State<AppState>,
      CurrentUser(current_user): CurrentUser,
      Path(task_id): Path<i32>,
      ValidatedJson(payload): ValidatedJson<TaskUpdate>,
) -> Result<Json<TaskOut>, ApiError> {
  payload.validate()?;

  let conn = state.pool.get().await?;
  get_task_or_404(&conn, task_id, current_user.id).await?;

  let changes = TaskChanges {
      title: payload.title,
      description: payload.description,
      status: payload.status,
      priority: payload.priority,
      due_date: payload.due_date,
      updated_at: Some(chrono::Utc::now()),
      };

      let task = conn
          .interact(move |c| {
                         diesel::update(tasks::table.filter(tasks::id.eq(task_id)))
                            .set(&changes)
                            .get_result::<Task>(c)
                            })
                            .await??;
                        Ok(Json(TaskOut::from(task)))
}
async fn delete_task(
      State(state): State<AppState>,
      CurrentUset(current_user): CurrentUser,
      Path(task_id): Path<i32>,
) -> Result<StatusCode, ApiError> {
  let conn = state.pool.get().await?;
  get_task_or_404(&conn, task_id, current_user.id).await?;

  conn.interact(move |c| {
                     diesel::delete(tasks::table.filter(tasks::id.eq(task_id))).execite(c)
                     })
                     .await??;

                    Ok(StatusCode::NO_CURRENT)
}
