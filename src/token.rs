use scrypt::{Params, scrypt};
use aes::Aes256;
use ctr::Ctr128BE;
use aes::cipher::{KeyIvInit, StreamCipher};
use rand::rngs::OsRng;
use rand::RngCore;
use hex;
use actix_web::HttpRequest;
use anyhow::Result;

pub struct Encryption {
    iv: [u8; 16],
    key: [u8; 32],
}

impl Encryption {
    pub fn new() -> Self {
        let mut iv = [0u8; 16];
        OsRng.fill_bytes(&mut iv);
        Encryption { iv, key: [0u8; 32] }
    }

    pub fn initialize(&mut self, password: &str) -> Result<()> {
        // scrypt with salt 'salt' (to match TS)
        let params = Params::recommended();
        let salt = b"salt";
        scrypt(password.as_bytes(), salt, &params, &mut self.key)?;
        Ok(())
    }

    pub fn encryption(&self, text: &str) -> Result<String> {
        type Aes256Ctr = Ctr128BE<Aes256>;
        let mut cipher = Aes256Ctr::new((&self.key).into(), (&self.iv).into());
        let mut buffer = text.as_bytes().to_vec();
        cipher.apply_keystream(&mut buffer);
        let encrypted_hex = hex::encode(buffer);
        Ok(encrypted_hex.chars().take(100).collect())
    }

    // decipher removed because tokens are single-direction in current flow
}

pub struct TokenService;

impl TokenService {
    #[allow(dead_code)]
    pub async fn generate_token(pool: &sqlx::Pool<sqlx::Mssql>, text_to_encrypt: &str, expired: bool, _time: Option<i64>, secret: &str) -> Result<String> {
        let mut enc = Encryption::new();
        enc.initialize(secret)?;
        let token = enc.encryption(&text_to_encrypt.repeat(20))?;
        // Determine expiredDate (DB expects a non-null datetime)
        let minutes = _time.unwrap_or(10);
        let expired_date = chrono::Utc::now() + chrono::Duration::minutes(minutes);
        let expired_date_str = expired_date.format("%Y-%m-%d %H:%M:%S").to_string();
        let expired_flag: i32 = if expired { 1 } else { 0 };

        // Parse user id from text_to_encrypt (handlers pass user.id.to_string())
        let user_id_parsed: i32 = text_to_encrypt.parse::<i32>().unwrap_or(0);

        // Upsert token: update if exists, otherwise insert and set metadata fields (usercrea/usermod/fechcrea/fechmod)
        let upsert_sql = r#"
            IF EXISTS (SELECT 1 FROM usertoken WHERE UserID = @p1)
            BEGIN
                UPDATE usertoken
                SET token = @p2,
                    createdDate = GETDATE(),
                    expiredDate = @p3,
                    expired = @p4,
                    usermod = @p5,
                    fechmod = GETDATE()
                WHERE UserID = @p1;
            END
            ELSE
            BEGIN
                INSERT INTO usertoken (UserID, token, createdDate, expiredDate, expired, usercrea, usermod, fechcrea, fechmod)
                VALUES (@p1, @p2, GETDATE(), @p3, @p4, @p5, @p5, GETDATE(), GETDATE());
            END
        "#;

        let _ = sqlx::query(upsert_sql)
            .bind(user_id_parsed)
            .bind(token.clone())
            .bind(expired_date_str)
            .bind(expired_flag)
            .bind(0i32) // usermod/usercrea default 0
            .execute(pool)
            .await?;
        Ok(token)
    }

    #[allow(dead_code)]
    pub async fn register_token(pool: &sqlx::Pool<sqlx::Mssql>, user_id: i64, token: &str) -> Result<sqlx::mssql::MssqlRow> {
        let expired_date = chrono::Utc::now() + chrono::Duration::minutes(10);
        let expired_date_str = expired_date.format("%Y-%m-%d %H:%M:%S").to_string();
        let upsert_sql = r#"
            IF EXISTS (SELECT 1 FROM usertoken WHERE UserID = @p1)
            BEGIN
                UPDATE usertoken
                SET token = @p2,
                    createdDate = GETDATE(),
                    expiredDate = @p3,
                    expired = @p4,
                    usermod = @p5,
                    fechmod = GETDATE()
                WHERE UserID = @p1;
                SELECT 1 as registered;
            END
            ELSE
            BEGIN
                INSERT INTO usertoken (UserID, token, createdDate, expiredDate, expired, usercrea, usermod, fechcrea, fechmod)
                VALUES (@p1, @p2, GETDATE(), @p3, @p4, @p5, @p5, GETDATE(), GETDATE());
                SELECT 1 as registered;
            END
        "#;

        let row = sqlx::query(upsert_sql)
            .bind(user_id as i32)
            .bind(token)
            .bind(expired_date_str)
            .bind(0i32)
            .bind(0i32)
            .fetch_one(pool)
            .await?;
        Ok(row)
    }

    #[allow(dead_code)]
    pub async fn validated_token(pool: &sqlx::Pool<sqlx::Mssql>, token: &str) -> Result<Vec<sqlx::mssql::MssqlRow>> {
        let sql = format!("EXEC SP_VALIDATE_TOKEN @token='{}'", token);
        let rows = sqlx::query(&sql).fetch_all(pool).await?;
        Ok(rows)
    }

    #[allow(dead_code)]
    pub fn extract_token_from_header(req: &HttpRequest) -> Option<String> {
        let header = req.headers().get("authorization")?.to_str().ok()?;
        let mut parts = header.split_whitespace();
        let typ = parts.next()?;
        let token = parts.next()?;
        if typ.eq_ignore_ascii_case("Bearer") { Some(token.to_string()) } else { None }
    }

    #[allow(dead_code)]
    pub async fn get_user_token(pool: &sqlx::Pool<sqlx::Mssql>, token: &str) -> Result<sqlx::mssql::MssqlRow> {
        let sql = format!("EXEC SP_GET_USER_TOKEN @token='{}'", token);
        let row = sqlx::query(&sql).fetch_one(pool).await?;
        Ok(row)
    }

    #[allow(dead_code)]
    pub async fn revoke_token(pool: &sqlx::Pool<sqlx::Mssql>, raw_token: &str) -> Result<sqlx::mssql::MssqlRow> {
        let sql = format!("EXEC SP_LOGOUT @token='{}'", raw_token);
        let row = sqlx::query(&sql).fetch_one(pool).await?;
        Ok(row)
    }
}
