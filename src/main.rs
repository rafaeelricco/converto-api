use server::run;
use db::mongodb::init_db_pool;
use log::{info, warn, error};
use std::env;

mod db;
mod controller;
mod models;
mod routes;
mod server;
mod utils;
mod ws;
mod file_processing;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match dotenv::dotenv() {
        Ok(_) => info!("Successfully loaded .env file"),
        Err(e) => warn!("Failed to load .env file: {}. Using default environment variables.", e),
    }

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info,actix_web=debug");
    }
    env_logger::init();
    info!("Logging initialized with RUST_LOG={}", env::var("RUST_LOG").unwrap_or_else(|_| "info,actix_web=debug".to_string()));

    let db_url = match env::var("DB_URL") {
        Ok(url) => url,
        Err(_) => {
            error!("Environment variable 'DB_URL' is not set. Please define it in your .env file.");
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "DB_URL environment variable not set"));
        }
    };

    info!("Initializing database connection pool...");
    let db_pool = match init_db_pool(&db_url).await {
        Ok(pool) => {
            pool
        },
        Err(e) => {
            error!("Failed to initialize MongoDB connection pool: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Database initialization error: {}", e)));
        }
    };

    let db = db_pool.database("rust-actix-web-mongodb");

    run(db)?.await
}
