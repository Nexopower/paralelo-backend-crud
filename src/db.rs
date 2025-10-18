use crate::models::{CreateUser, UpdateUser, User};
use sqlx::{Pool, Mssql, mssql::MssqlPoolOptions, Transaction, Row};
use bcrypt::{hash, DEFAULT_COST};
use anyhow::Result;

pub async fn init_db(settings: &crate::config::Settings) -> Result<Pool<Mssql>> {
    // Build connection string for MSSQL
    let user = settings.db.user.clone().unwrap_or_default();
    let password = settings.db.password.clone().unwrap_or_default();
    let host = settings.db.host.clone().unwrap_or_else(|| "127.0.0.1".into());
    let port = settings.db.port;
    let database = settings.db.database.clone().unwrap_or_default();
    let conn = format!("mssql://{}:{}@{}:{}/{}", user, password, host, port, database);

            let pool = MssqlPoolOptions::new().connect(&conn).await?;

            Ok(pool)
}

#[allow(dead_code)]
pub async fn execute_query(pool: &Pool<Mssql>, query: &str) -> Result<sqlx::mssql::MssqlRow> {
    let row = sqlx::query(query).fetch_one(pool).await?;
    Ok(row)
}
#[allow(dead_code)]
pub async fn execute_query_params<T: serde::Serialize + Send + Sync>(pool: &Pool<Mssql>, query: &str, _params: &T) -> Result<sqlx::mssql::MssqlRow> {
    // sqlx for mssql does not support generic param binding from map; use simple approach: caller should use proper query with args.
    // For compatibility, we will just execute raw query assuming params are already embedded or use format!. Use a wrapper in handlers.
    let row = sqlx::query(query).fetch_one(pool).await?;
    Ok(row)
}

#[allow(dead_code)]
pub async fn begin_transaction(pool: &Pool<Mssql>) -> Result<Transaction<'_, Mssql>> {
    let tx = pool.begin().await?;
    Ok(tx)
}
#[allow(dead_code)]
pub async fn commit_transaction(tx: Transaction<'_, Mssql>) -> Result<()> {
    tx.commit().await?;
    Ok(())
}
#[allow(dead_code)]
pub async fn rollback_transaction(tx: Transaction<'_, Mssql>) -> Result<()> {
    tx.rollback().await?;
    Ok(())
}
#[allow(non_snake_case)]
#[allow(dead_code)]
pub async fn getdate(pool: &Pool<Mssql>) -> Result<String> {
    let row = sqlx::query("SELECT CONVERT(varchar, GETDATE(), 120) as DATE").fetch_one(pool).await?;
    let dt: String = row.try_get("DATE")?;
    Ok(dt)
}

// Basic user CRUD using MSSQL stored procedures or inline queries
pub async fn create_user(pool: &Pool<Mssql>, input: CreateUser) -> Result<User> {
    let password_hash = hash(&input.password, DEFAULT_COST)?;
    // Call stored procedure sp_usuarios_insert (signature: nombre, email, codperf, contrasena, usercrea, usermod, fechcrea, fechmod)
    let _ = sqlx::query(
        "EXEC sp_usuarios_insert @nombre_usr = @p1, @email_usr = @p2, @codperf_usr = @p3, @contrasena_usr = @p4, @usercrea = @p5, @usermod = @p6, @fechcrea = @p7, @fechmod = @p8"
    )
    .bind(input.username.clone())
    .bind(input.email.clone().unwrap_or_default())
    .bind(1i32)
    .bind(password_hash)
    .bind(0i32)
    .bind(0i32)
    .bind(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
    .bind(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
    .execute(pool)
    .await?;

    // Return the created user by selecting last inserted by name (assuming unique nombre_usr)
    let rec = sqlx::query_as::<_, User>(
        "SELECT codusr_usr, nombre_usr, email_usr, contrasena_usr FROM usuarios WHERE nombre_usr = @p1"
    )
    .bind(input.username)
    .fetch_one(pool)
    .await?;
    Ok(rec)
}

pub async fn list_users(pool: &Pool<Mssql>) -> Result<Vec<User>> {
    let users = sqlx::query_as::<_, User>("SELECT codusr_usr, nombre_usr, email_usr, contrasena_usr FROM usuarios").fetch_all(pool).await?;
    Ok(users)
}

pub async fn find_by_username(pool: &Pool<Mssql>, username: &str) -> Result<Option<User>> {
    let u = sqlx::query_as::<_, User>("SELECT codusr_usr, nombre_usr, email_usr, contrasena_usr FROM usuarios WHERE nombre_usr = @p1 OR email_usr = @p1")
        .bind(username)
        .fetch_optional(pool)
        .await?;
    Ok(u)
}

pub async fn get_user(pool: &Pool<Mssql>, user_id: i32) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT codusr_usr, nombre_usr, email_usr, contrasena_usr FROM usuarios WHERE codusr_usr = @p1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn update_user(pool: &Pool<Mssql>, user_id: i32, input: UpdateUser) -> Result<Option<User>> {
    // Use the stored procedure sp_usuarios_update if available
    let current = get_user(pool, user_id).await?;
    if current.is_none() {
        return Ok(None);
    }
    let cur = current.unwrap();
    let new_username = input.username.unwrap_or(cur.username);
    let new_email = input.email.unwrap_or(cur.email.unwrap_or_default()); // Silence unused new_email
    let new_password = if let Some(pw) = input.password { hash(&pw, DEFAULT_COST)? } else { cur.password_hash };

    // Updated sp_usuarios_update signature: @codusr_usr, @nombre_usr, @email_usr, @codperf_usr, @contrasena_usr, @usercrea, @usermod, @fechcrea, @fechmod
    let _ = sqlx::query(
        "EXEC sp_usuarios_update @codusr_usr = @p1, @nombre_usr = @p2, @email_usr = @p3, @codperf_usr = @p4, @contrasena_usr = @p5, @usercrea = @p6, @usermod = @p7, @fechcrea = @p8, @fechmod = @p9"
    )
    .bind(user_id)
    .bind(new_username)
    .bind(new_email.clone())
    .bind(1i32)
    .bind(new_password)
    .bind(0i32)
    .bind(0i32)
    .bind(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
    .bind(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
    .execute(pool)
    .await?;
    Ok(get_user(pool, user_id).await?)
}

pub async fn delete_user(pool: &Pool<Mssql>, user_id: i32) -> Result<bool> {
    let _ = sqlx::query("EXEC sp_usuarios_delete @codusr_usr = @p1").bind(user_id).execute(pool).await?;
    Ok(true)
}

