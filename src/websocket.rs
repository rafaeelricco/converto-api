use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
use actix_web::web;
use actix_web_actors::ws;
use log::{info,error,debug};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;
use serde_json::json;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebSocketConnection {
    id: String,
    hb: Instant,
    states: Arc<Mutex<HashMap<String, String>>>,
}

impl Actor for WebSocketConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                debug!("Ping received for id: {}", self.id);
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                debug!("Pong received for id: {}", self.id);
                self.hb = Instant::now();
            }
            Ok(ws::Message::Continuation(_)) => {
            }
            Ok(ws::Message::Nop) => {
            }
            Ok(ws::Message::Text(text)) => {
                info!("Text message received for id {}: {}", self.id, text);
                let response = format!("Received message for connection {}: {}", self.id, text);
                ctx.text(response);
            }
            Ok(ws::Message::Binary(bin)) => {
                info!("Binary message received for id: {}", self.id);
                ctx.binary(bin);
            }
            Ok(ws::Message::Close(reason)) => {
                info!("Close message received for id: {}", self.id);
                ctx.close(reason);
                ctx.stop();
            }
            Err(e) => {
                error!("Error in WebSocket for id {}: {:?}", self.id, e);
                ctx.stop();
            }
        }
    }
}

impl WebSocketConnection {
    pub fn new(states: Arc<Mutex<HashMap<String, String>>>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            hb: Instant::now(),
            states,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateState {
    pub id: String,
    pub state: String,
}

impl Handler<UpdateState> for WebSocketConnection {
    type Result = ();

    fn handle(&mut self, msg: UpdateState, ctx: &mut Self::Context) {
        let mut states = self.states.lock().unwrap();
        states.insert(msg.id.clone(), msg.state.clone());
        ctx.text(json!({
            "id": msg.id,
            "state": msg.state,
        }).to_string());
    }
}


pub async fn update_websocket_state(
    states: web::Data<Arc<Mutex<HashMap<String, String>>>>,
    id: String,
    new_state: String,
) {
    let mut states = states.lock().unwrap();
    states.insert(id, new_state);
}
