// use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
// use actix_web::web;
// use actix_web_actors::ws;
// use log::{info,error,debug};
// use std::collections::HashMap;
// use std::sync::{Arc, Mutex};
// use std::time::{Duration, Instant};
// use uuid::Uuid;
// use serde_json::json;

// const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
// const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// pub struct WebSocketConnection {
//     id: String,
//     hb: Instant,
//     states: Arc<Mutex<HashMap<String, String>>>,
// }

// impl Actor for WebSocketConnection {
//     type Context = ws::WebsocketContext<Self>;

//     fn started(&mut self, ctx: &mut Self::Context) {
//         self.hb(ctx);
//     }
// }

// impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketConnection {
//     fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
//         match msg {
//             Ok(ws::Message::Ping(msg)) => {
//                 debug!("Ping received for id: {}", self.id);
//                 self.hb = Instant::now();
//                 ctx.pong(&msg);
//             }
//             Ok(ws::Message::Pong(_)) => {
//                 debug!("Pong received for id: {}", self.id);
//                 self.hb = Instant::now();
//             }
//             Ok(ws::Message::Continuation(_)) => {
//             }
//             Ok(ws::Message::Nop) => {
//             }
//             Ok(ws::Message::Text(text)) => {
//                 info!("Text message received for id {}: {}", self.id, text);
//                 let response = format!("Received message for connection {}: {}", self.id, text);
//                 ctx.text(response);
//             }
//             Ok(ws::Message::Binary(bin)) => {
//                 info!("Binary message received for id: {}", self.id);
//                 ctx.binary(bin);
//             }
//             Ok(ws::Message::Close(reason)) => {
//                 info!("Close message received for id: {}", self.id);
//                 ctx.close(reason);
//                 ctx.stop();
//             }
//             Err(e) => {
//                 error!("Error in WebSocket for id {}: {:?}", self.id, e);
//                 ctx.stop();
//             }
//         }
//     }
// }

// impl WebSocketConnection {
//     pub fn new(states: Arc<Mutex<HashMap<String, String>>>) -> Self {
//         let id = Uuid::new_v4().to_string();
//         {
//             let mut states_lock = states.lock().unwrap();
//             states_lock.insert(id.clone(), "Iniciada conexão com servidor Converto.".to_string());
//         }
//         Self {
//             id,
//             hb: Instant::now(),
//             states: states.clone(),
//         }
//     }

//     fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
//         ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
//             if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
//                 println!("Websocket Client heartbeat failed, disconnecting!");
//                 ctx.stop();
//                 return;
//             }
//             ctx.ping(b"");
//         });
//     }
// }


// pub async fn update_websocket_state(id: String, message: String) {
//     let states = web::Data::new(Arc::new(Mutex::new(HashMap::<String, String>::new())));
//     let mut states_lock = states.lock().unwrap();
//     states_lock.insert(id.clone(), message.clone());
// }
