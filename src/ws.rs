use actix::{Actor, Running, StreamHandler, Handler, Message, AsyncContext, ActorContext};
use actix::prelude::Recipient;
use actix_multipart::form::text::Text;
use actix_web_actors::ws;
use uuid::Uuid;
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct State {
    progress: f32,
    status: String,
}

pub struct WsConn {
    pub id: Uuid,
    state: State,
    hb: Instant,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

impl WsConn {
    pub fn new() -> Self {
        WsConn {
            id: Uuid::new_v4(),
            state: State {
                progress: 0.0,
                status: "Connecting".to_string(),
            },
            hb: Instant::now(),
        }
    }

    fn send_status(&self, ctx: &mut ws::WebsocketContext<Self>) {
        let status_message = format!(
            "{{\"id\": \"{}\", \"progress\": {}, \"status\": \"{}\"}}",
            self.id, self.state.progress, self.state.status
        );
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
        self.state.status = "Connected".to_string();
        self.send_status(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.state.status = "Disconnected".to_string();
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
