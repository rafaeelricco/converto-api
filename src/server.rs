use crate::ws::*;
use crate::compress::websocket::*;
use crate::compress::pdf::{post_compress_pdf, CompressionLevel};

use actix_web::dev::Server;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder, Error as ActixError};
use actix_web::middleware::Logger;
use actix::prelude::*;

use serde::{Deserialize, Serialize};
use mongodb::Database;
use chrono::Utc;
use log::info;
use uuid::Uuid;
use std::env;
use std::sync::atomic::AtomicUsize;
use actix_web_actors::ws::WsResponseBuilder;


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

// async fn ws_route(
//     req: HttpRequest,
//     stream: web::Payload,
//     srv: web::Data<Addr<FileProcessor>>,
// ) -> Result<HttpResponse, ActixError> {
//     let ws = WsConn::new();
//     let id = ws.id;
//     let file_processor_addr = srv.get_ref().clone();

//     let resp = WsResponseBuilder::new(ws, &req, stream).start_with_addr()?;
//     file_processor_addr.send(AddSession { id, addr: resp.0.recipient()  }).await.unwrap();
//     Ok(resp.1)
// }

async fn ws_test_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<FileProcessor>>,
) -> Result<HttpResponse, ActixError> {
    let id = Uuid::parse_str("test-id").unwrap();
    let file_processor_addr = srv.get_ref().clone();

    let resp = WsResponseBuilder::new(WsConn::new(id), &req, stream).start_with_addr()?;
    file_processor_addr.send(AddSession { id, addr: resp.0.recipient() }).await.unwrap();
    Ok(resp.1)
}

async fn send_test_message(
    srv: web::Data<Addr<FileProcessor>>,
) -> impl Responder {
    let test_id = "test-id";
    srv.send(UpdateProgress { 
        id: Uuid::parse_str(test_id).unwrap(),
files: vec![FileProgress {
    id: Uuid::new_v4().to_string(),
    progress: 0.0,
    message: "Test message".to_string(),
    file_name: None,
    compression_level: Some(format!("{:?}", CompressionLevel::Medium)), 
}],
        status: Status::InProgress,
    }).await.unwrap();
    HttpResponse::Ok().body(format!("Test message sent with ID: {}", test_id))
}

#[derive(Deserialize)]
struct WsParams { id: String, }

async fn ws_with_id_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<FileProcessor>>,
    params: web::Query<WsParams>,
) -> Result<HttpResponse, ActixError> {
    let id = Uuid::parse_str(&params.id).unwrap_or_else(|_| Uuid::new_v4());
    let ws = WsConn::new(id);
    let file_processor_addr = srv.get_ref().clone();

    let resp = WsResponseBuilder::new(ws, &req, stream).start_with_addr()?;
    file_processor_addr.send(AddSession { id, addr: resp.0.recipient() }).await.unwrap();
    Ok(resp.1)
}


pub fn run(db: Database) -> Result<Server, std::io::Error> {
    info!("Starting server...");
    
    let db = web::Data::new(db);
    let app_state = web::Data::new(AtomicUsize::new(0));
    let processor = FileProcessor::new().start();
    
    let bind_address = if env::var("ENV").unwrap_or("dev".to_string()) == "dev" {
        "127.0.0.1:10000"
    } else {
        "0.0.0.0:10000"
    };

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::PayloadConfig::new(1024 * 1024 * 50))
            .app_data(db.clone())
            .app_data(app_state.clone())
            .app_data(web::Data::new(processor.clone()))
            .route("/", web::get().to(root))
            .route("/ws", web::get().to(ws_with_id_route))
            .route("/ws_test", web::get().to(ws_test_route))
            .route("/send_test_message", web::post().to(send_test_message))
            .route("/compress", web::post().to(post_compress_pdf))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(bind_address)?
    .run();

    info!("Server running at http://{}", bind_address);
    info!("WebSocket server running at ws://{}", bind_address);
    info!("Press 'Ctrl + C' to stop the server");
    Ok(server)
}
