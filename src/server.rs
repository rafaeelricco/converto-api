// use crate::routes::notes;
// use crate::routes::users;
use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use mongodb::Database;
use serde::Serialize;
use std::net::TcpListener;

#[derive(Serialize)]
struct ApiInfo {
    api: &'static str,
    version: &'static str,
    database: Option<String>,
    date_created: String,
}

async fn root() -> impl Responder {
    const API_VERSION: &str = env!("CARGO_PKG_VERSION");
    const API_NAME: &str = env!("CARGO_PKG_NAME");

    let date_created: String = Utc::now().format("%d-%m-%Y").to_string();

    let api_infos: ApiInfo = ApiInfo {
        api: API_NAME,
        version: API_VERSION,
        date_created: date_created,
        database: None,
    };

    HttpResponse::Ok().json(api_infos)
}

pub fn run(listener: TcpListener, db: Database) -> Result<Server, std::io::Error> {
    let db = web::Data::new(db);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(db.clone())
            .route("/", web::get().to(root))
            // .configure(notes::configure_note_routes)
            // .configure(users::configure_auth_routes)
    })
    .listen(listener)?
    .run();
    Ok(server)
}
