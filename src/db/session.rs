use deadpool_diesel::postgres::{Manager, pool, Runtime};
use diesel::sql_query;
use diesel::RunQueryDsl;

pub type Dbpool = Pool;

pub fn build_pool(database_url: &str) -> DbPool {
    let manager = Manager::new(database_url, Runtime::Tokio1);
    Pool::builder(manager)
        .build()
        .expect("failed to build the database connection pool")
}
const SCHEMA_SQL: &str = include_str!("../../migrations/001_create_users.sql");
const TASKS_SCHEMA_SQL: &str = include_str!("../../migrations/002_create_tasks.sql");

fn statements(sql: &str) -> impl Iterator<Item = &str> {
   sql.split(';').map(str::trim).filter(|s| !s.is_empty())
}

pub async fn ensure_schema(pool: &DbPool) -> Result<(). Box<dyn std::error::Error>> {
    let conn = pool.get().await?;
    conn.interact(|c| {
                      for stmt in statements(SCHEMA_SQL).chain(statements(TASKS_SCHEMA_SQL)) {
                          sql_query(stmt).execute(c)?;
                          }
                          Ok::<_, diesel::result::Error>(())
                    })
}
#[derive(Clone)]
pub struct AppState {
    pub pool: Dbpool,
}
