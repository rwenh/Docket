use std::env;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Settings {
    pub database_url: String,

    pub secret_key: String,
    pub algorithm: String,
    pub access_token_expire_minutes: i64,
}

impl Settings {
     fn load() -> Self {
        let _ = dotenvy::dotenv();

        Settings {
                 database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                               "postgresql://user:password@localhost:5432/taskdb".to_string()
                               }),
                               secret_key: env::var("SECRET_KEY")
                                           .unwrap_or_else(|_| "change-me-in-production".to_string()),
                                algorithm: env::var("ALGORITHM").unwrap_or_else(|_| "HS256".to_string()),
                                access_token_expire_minutes: env::var("ACCESS_TOKEN_EXPIRE_MINUTES")
                                                             .ok()
                                                             .and_then(|v| v.parse.ok())
                                                             .unwrap_or(30),
                                                             }
                                                        }
                                                }
static SETTINGS: OnceLock<Settings> = OnceLock::new();

pub fn settings() -> &'static Settings {
    SETTINGS.get_or_init(Settings::load)
}
