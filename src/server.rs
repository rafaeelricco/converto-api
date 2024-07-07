use actix_web::dev::Server;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use chrono::Utc;
use mongodb::Database;
use serde::Serialize;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use log::{info, error};

use crate::routes::pdf::configure_pdf_routes;
// use crate::websocket::WebSocketConnection;


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

// async fn websocket(
//     req: HttpRequest,
//     stream: web::Payload,
//     states: web::Data<Arc<Mutex<HashMap<String, String>>>>,
// ) -> Result<HttpResponse, actix_web::Error> {
//     info!("WebSocket connection attempt");
//     let ws = WebSocketConnection::new(states.get_ref().clone());
//     match ws::start(ws, &req, stream) {
//         Ok(response) => {
//             info!("WebSocket connection established successfully");
//             Ok(response)
//         },
//         Err(e) => {
//             error!("Failed to establish WebSocket connection: {:?}", e);
//             Err(e)
//         }
//     }
// }


pub fn run(listener: TcpListener, db: Database) -> Result<Server, std::io::Error> {
    info!("Starting server...");
    let db = web::Data::new(db);
    let states = web::Data::new(Arc::new(Mutex::new(
        HashMap::<String, String>::new(),
    )));

    let local_addr = listener.local_addr()?;

    let server = HttpServer::new(move || {
        App::new()
        .app_data(web::PayloadConfig::new(1024 * 1024 * 50)) 
            .app_data(db.clone())
            .app_data(states.clone())
            .route("/", web::get().to(root))
            // .route("/ws", web::get().to(websocket))
            .configure(configure_pdf_routes)
    })
    .listen(listener)?
    .run();

    info!("Server running at http://{}", local_addr);
    info!("WebSocket server running at ws://{}/ws", local_addr);
    info!("Press 'Ctrl + C' to stop the server");
    Ok(server)
}
