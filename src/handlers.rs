use actix_web::{web, HttpResponse, Responder};
use sqlx::Pool;
use sqlx::Mssql;
use crate::models::{CreateUser, LoginRequest, LoginResponse, UpdateUser};
use crate::db;
use crate::token::TokenService;
use bcrypt::verify;
use futures::stream::{self, StreamExt};
use futures::future::try_join_all;
use std::time::Duration;
use tokio::time::timeout;

pub async fn login(pool: web::Data<Pool<Mssql>>, cfg: web::Data<crate::config::Settings>, body: web::Json<LoginRequest>) -> impl Responder {
    match db::find_by_username(&pool, &body.username).await {
        Ok(Some(user)) => {
            if verify(&body.password, &user.password_hash).unwrap_or(false) {
                match TokenService::generate_token(&pool, &user.id.to_string(), false, None, &cfg.jwt_secret).await {
                    Ok(token) => HttpResponse::Ok().json(LoginResponse { token }),
                    Err(e) => {
                        eprintln!("token gen err: {}", e);
                        HttpResponse::InternalServerError().finish()
                    }
                }
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("Invalid credentials"),
        Err(e) => {
            eprintln!("db err: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_user(pool: web::Data<Pool<Mssql>>, body: web::Json<CreateUser>) -> impl Responder {
    match db::create_user(&pool, body.0).await {
        Ok(user) => HttpResponse::Created().json(user),
        Err(e) => HttpResponse::BadRequest().body(format!("Err: {}", e)),
    }
}

pub async fn list_users(pool: web::Data<Pool<Mssql>>) -> impl Responder {
    match db::list_users(&pool).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_user(pool: web::Data<Pool<Mssql>>, path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    match db::get_user(&pool, id).await {
        Ok(Some(u)) => HttpResponse::Ok().json(u),
        Ok(None) => HttpResponse::NotFound().body("Not found"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_user(pool: web::Data<Pool<Mssql>>, path: web::Path<i32>, body: web::Json<UpdateUser>) -> impl Responder {
    let id = path.into_inner();
    match db::update_user(&pool, id, body.0).await {
        Ok(Some(u)) => HttpResponse::Ok().json(u),
        Ok(None) => HttpResponse::NotFound().body("Not found"),
        Err(e) => HttpResponse::BadRequest().body(format!("Err: {}", e)),
    }
}

pub async fn delete_user(pool: web::Data<Pool<Mssql>>, path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();
    match db::delete_user(&pool, id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().body("Not found"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Example endpoint demonstrating concurrent data load using join_all (Promise.all equivalent)
pub async fn load_concurrent(pool: web::Data<Pool<Mssql>>, cfg: web::Data<crate::config::Settings>) -> impl Responder {
    // For demo, we will fetch the list of users and then fetch each user individually concurrently
    match db::list_users(&pool).await {
        Ok(users) => {
            let concurrency_limit = cfg.concurrency_limit;
            let timeout_secs = cfg.db_query_timeout_secs;
            let fail_fast = cfg.fail_fast;

            // Build a vector of futures where each db::get_user is wrapped with a timeout
            let futures_vec = users.into_iter().map(|u| {
                let pool = pool.clone();
                async move {
                    // apply per-query timeout
                    let fut = db::get_user(&pool, u.id);
                    match timeout(Duration::from_secs(timeout_secs), fut).await {
                        Ok(inner_res) => inner_res, // Result<Option<User>, sqlx::Error>
                            Err(_) => Err(anyhow::anyhow!("timeout")),
                    }
                }
            }).collect::<Vec<_>>();

            // If fail_fast is desired, use try_join_all which returns Err on first Err.
            if fail_fast {
                match try_join_all(futures_vec).await {
                    Ok(results) => {
                        let mut out = Vec::new();
                        for r in results {
                            if let Some(u) = r {
                                out.push(u);
                            }
                        }
                        return HttpResponse::Ok().json(out);
                    }
                    Err(e) => return HttpResponse::InternalServerError().body(format!("Err: {}", e)),
                }
            }

            // Otherwise run with limited concurrency using buffer_unordered
            let stream = stream::iter(futures_vec.into_iter().map(|fut| async { fut.await }));
            let results: Vec<_> = stream.buffer_unordered(concurrency_limit).collect().await;

            // flatten Option<User>
            let mut out = Vec::new();
            for r in results {
                if let Ok(Some(u)) = r {
                    out.push(u);
                }
            }
            HttpResponse::Ok().json(out)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
