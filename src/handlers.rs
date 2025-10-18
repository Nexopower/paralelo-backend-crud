use actix_web::{web, HttpResponse, Responder};
use sqlx::Pool;
use sqlx::Mssql;
use crate::models::{CreateUser, LoginRequest, LoginResponse, UpdateUser};
use crate::db;
use crate::token::TokenService;
use bcrypt::verify;
use futures::future::join_all;

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
pub async fn load_concurrent(pool: web::Data<Pool<Mssql>>) -> impl Responder {
    // For demo, we will fetch the list of users and then fetch each user individually concurrently
    match db::list_users(&pool).await {
        Ok(users) => {
            let futures = users.iter().map(|u| db::get_user(&pool, u.id));
            let results: Vec<_> = join_all(futures).await;
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
