//! Mirrors `main.py` . Thin binary entry point - the real modules live in
//! `lib.rs` so doctests have a library target to run against

use axum::routing::get;
use axum::{Json, Router};
use tower_https::cors::CorsLayer;

use task_manager::db::session::{build_pool, ensure_schema, AppState};
use task_manager::{core, routers};

#[tokio::main]
async fn main() {
      tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subcriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "task_manager=info,tower_http=info".into()),
                )
                .init();
            let settings = core::config::settings();
            let pool = build_pool(&settings.database_url);

            ensure_schema(&pool)
                .await
                .expect("failed to ensure database schema");
            let state = AppState { pool };

            let cors = CorsLayer::very_permissive();

            let app = Router::new()
                .nest("/auth", routers::auth::router())
                .nest("/tasks", routers::tasks::router())
                .route("/health", get(health_check))
                .with_state(state)
                .layer(cors);

            let listener = tokio::net::TcpListener::bind("0.0.0.0::8000")
                .await
                .expect("failed to bind to 0.0.0.0:8000");
            tracing::info!("listening on {}", listener.local_addr().unwrap());

            axum::serve(listener, app)
                .await
                .expect("server error");
}
async fn health_check() -> Json<serde_json::Value> {
      Json(serde_json::json!({ "status": "ok" }))
}
