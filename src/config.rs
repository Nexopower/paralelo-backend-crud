use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
pub struct DbSettings {
    pub user: Option<String>,
    pub password: Option<String>,
    pub host: Option<String>,
    pub port: u16,
    pub database: Option<String>,
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout_secs: u64,
    pub encrypt: bool,
    pub trust_server_certificate: bool,
}

#[derive(Clone)]
pub struct Settings {
    pub db: DbSettings,
    pub port: u16,
    pub jwt_secret: String,
    pub concurrency_limit: usize,
    pub db_query_timeout_secs: u64,
    pub fail_fast: bool,
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
            min_connections: env::var("DB_MIN_CONNECTIONS").ok().and_then(|s| s.parse().ok()).unwrap_or(1),
            max_connections: env::var("DB_MAX_CONNECTIONS").ok().and_then(|s| s.parse().ok()).unwrap_or(10),
            acquire_timeout_secs: env::var("DB_ACQUIRE_TIMEOUT_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(30),
            encrypt: env::var("DB_ENCRYPT").ok().map(|s| matches!(s.as_str(), "1" | "true" | "True" | "yes")).unwrap_or(true),
            trust_server_certificate: env::var("DB_TRUST_SERVER_CERT").ok().map(|s| matches!(s.as_str(), "1" | "true" | "True" | "yes")).unwrap_or(true),
        };
        let port = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret123".into());
        let concurrency_limit = env::var("CONCURRENCY_LIMIT").ok().and_then(|s| s.parse().ok()).unwrap_or(20usize);
        let db_query_timeout_secs = env::var("DB_QUERY_TIMEOUT_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(5u64);
        let fail_fast = env::var("FAIL_FAST").ok().map(|s| matches!(s.as_str(), "1" | "true" | "True" | "yes")).unwrap_or(false);
        Settings { db, port, jwt_secret, concurrency_limit, db_query_timeout_secs, fail_fast }
    }
}

