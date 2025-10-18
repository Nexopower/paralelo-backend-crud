use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
pub struct DbSettings {
    pub user: Option<String>,
    pub password: Option<String>,
    pub host: Option<String>,
    pub port: u16,
    pub database: Option<String>,
}

#[derive(Clone)]
pub struct Settings {
    pub db: DbSettings,
    pub port: u16,
    pub jwt_secret: String,
}

impl Settings {
    pub fn from_env() -> Self {
        dotenv().ok();
        let db = DbSettings {
            user: env::var("DATABASE_USERNAME").ok(),
            password: env::var("DATABASE_PASSWORD").ok(),
            host: env::var("DATABASE_HOST").ok(),
            port: env::var("DATABASE_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(1433),
            database: env::var("DATABASE_NAME").ok(),
        };
        let port = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret123".into());
        Settings { db, port, jwt_secret }
    }
}

