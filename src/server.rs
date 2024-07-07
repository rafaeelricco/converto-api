use actix::Actor;
use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use mongodb::Database;
use serde::Serialize;
use std::net::TcpListener;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use log::info;

use crate::{session, websocket};

use std::{
    sync::{
        atomic::Ordering,
    },
    time::Instant,
};

use actix::*;
use actix_files::{Files, NamedFile};
use actix_web::{
    middleware::Logger, Error, HttpRequest,
};
use actix_web_actors::ws;




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


async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<websocket::ChatServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        session::WsChatSession {
            id: 0,
            hb: Instant::now(),
            room: "main".to_owned(),
            name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

/// Displays state
async fn get_count(count: web::Data<AtomicUsize>) -> impl Responder {
    let current_count = count.load(Ordering::SeqCst);
    format!("Visitors: {current_count}")
}


pub fn run(listener: TcpListener, db: Database) -> Result<Server, std::io::Error> {
    info!("Starting server...");
    let db = web::Data::new(db);

    let app_state = Arc::new(AtomicUsize::new(0));
    let server = websocket::ChatServer::new(app_state.clone()).start();

    let server = HttpServer::new(move || {
        App::new()
        .app_data(web::PayloadConfig::new(1024 * 1024 * 50))
        .app_data(db.clone())
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(server.clone()))
            .route("/count", web::get().to(get_count))
            .route("/ws", web::get().to(chat_route))
            .route("/", web::get().to(root))
            .service(Files::new("/static", "./static"))
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
