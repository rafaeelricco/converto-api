use actix::{Actor, Running, StreamHandler, Handler, Message, AsyncContext, ActorContext};
use actix_web_actors::ws;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Connecting,
    Connected,
    InProgress,
    Completed,
    Disconnected,
}

#[derive(Debug, Serialize, Clone)]
pub struct FileProgress {
    pub id: String,
    pub progress: f32,
    pub file_name: Option<String>,
    pub message: String,
    pub compression_level: Option<String>,  
}

#[derive(Debug)]
pub struct WsConn {
    pub id: Uuid,
    pub files: Vec<FileProgress>,
    pub status: Status,
    hb: Instant,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

impl WsConn {
    pub fn new(id: Uuid) -> Self {
        WsConn {
            id,
            files: Vec::new(),
            hb: Instant::now(),
            status: Status::Connecting,
        }
    }

    fn send_status(&self, ctx: &mut ws::WebsocketContext<Self>) {
        let status_message = json!({
            "id": self.id,
            "files": self.files,
            "status": self.status,
        }).to_string();
        ctx.text(status_message);
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting failed heartbeat");
                ctx.stop();
                return;
            }
            ctx.ping(b"hi");
        });
    }
}

impl Actor for WsConn {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.status = Status::Connected;
        self.send_status(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.status= Status::Disconnected;
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(ws::Message::Text(text)) => {
                println!("Received text: {}", text);
                ctx.text(text);
            }
            Err(e) => std::panic::panic_any(e),
        }
    }
}

impl Handler<WsMessage> for WsConn {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
