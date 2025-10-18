mod config;
mod models;
mod db;
mod auth;
mod token;
mod handlers;

use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = config::Settings::from_env();
    println!("Using DB host: {:?}", settings.db.host);

    let pool = match db::init_db(&settings).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to init db: {}", e);
            std::process::exit(1);
        }
    };

    let data_pool = web::Data::new(pool);
    let data_cfg = web::Data::new(settings.clone());

    let bind_addr = format!("0.0.0.0:{}", settings.port);

    HttpServer::new(move || {
        App::new()
            .app_data(data_pool.clone())
            .app_data(data_cfg.clone())
            .route("/login", web::post().to(handlers::login))
            .route("/users", web::post().to(handlers::create_user))
            .route("/users", web::get().to(handlers::list_users))
            .route("/users/{id}", web::get().to(handlers::get_user))
            .route("/users/{id}", web::put().to(handlers::update_user))
            .route("/users/{id}", web::delete().to(handlers::delete_user))
            .route("/load_concurrent", web::get().to(handlers::load_concurrent))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
