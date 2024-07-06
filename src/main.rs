use db::mongodb::init_db_pool;
use log::info;
use server::run;
use std::{env, net::TcpListener};

mod db;
mod controller;
mod models;
mod routes;
mod server;
mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env::set_var("RUST_LOG", "info,actix_web=debug");
    env_logger::init();
    info!("Starting server...");

    let address = dotenv::var("HOST").expect("A variável de ambiente 'address' não está definida. Por favor, defina-a no seu arquivo .env.");
    let db_url = dotenv::var("DB_URL").expect("A variável de ambiente 'DB_URL' não está definida. Por favor, defina-a no seu arquivo .env.");

    let listener = TcpListener::bind(address.clone()).expect("Failed to bind to the listener");

    let db_pool = init_db_pool(&db_url)
        .await
        .expect("Erro ao inicializar o pool de conexões do MongoDB.");
    let db = db_pool.database("rust-actix-web-mongodb");

    info!("Starting server at http://{}", address);
    run(listener, db)?.await
}
