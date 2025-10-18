use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug, Clone)]
pub struct User {
    #[sqlx(rename = "codusr_usr")]
    pub id: i32,
    #[sqlx(rename = "nombre_usr")]
    pub username: String,
    #[sqlx(rename = "email_usr")]
    pub email: Option<String>,
    #[sqlx(rename = "contrasena_usr")]
    pub password_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}
