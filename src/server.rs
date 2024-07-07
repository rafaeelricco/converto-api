use actix_web::dev::Server;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder, Error as ActixError};
use actix_web::middleware::Logger;
use actix::prelude::*;
use mongodb::Database;
use serde::Serialize;
use chrono::Utc;
use log::info;
use std::sync::atomic::AtomicUsize;
use actix_web_actors::ws::WsResponseBuilder;

use crate::ws::WsConn;
use crate::file_processing::{FileProcessor, AddSession};

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

async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<FileProcessor>>,
) -> Result<HttpResponse, ActixError> {
    let ws = WsConn::new();
    let id = ws.id;
    let file_processor_addr = srv.get_ref().clone();

    let resp = WsResponseBuilder::new(ws, &req, stream).start_with_addr()?;
    file_processor_addr.send(AddSession { id, addr: resp.0.recipient()  }).await.unwrap();
    Ok(resp.1)
}

pub fn run(db: Database) -> Result<Server, std::io::Error> {
    info!("Starting server...");
    let db = web::Data::new(db);

    let app_state = web::Data::new(AtomicUsize::new(0));
    let processor = FileProcessor::new().start();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::PayloadConfig::new(1024 * 1024 * 50))
            .app_data(db.clone())
            .app_data(app_state.clone())
            .app_data(web::Data::new(processor.clone()))
            .route("/", web::get().to(root))
            .route("/ws", web::get().to(ws_route))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("127.0.0.1", 5000))?
    .run();

    info!("Server running at http://127.0.0.1:5000");
    info!("WebSocket server running at ws://127.0.0.1:5000/ws");
    info!("Press 'Ctrl + C' to stop the server");
    Ok(server)
}
