use actix::{Actor, Context, Handler, Message, MessageResult, Recipient};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use futures::lock::Mutex;
use crate::ws::WsMessage;

type Socket = Recipient<WsMessage>;

#[derive(Clone)]
pub struct FileProcessor {
    sessions: Arc<Mutex<HashMap<Uuid, Socket>>>,
}

impl FileProcessor {
    pub fn new() -> Self {
        FileProcessor {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_session(&self, id: Uuid, addr: Socket) {
        let mut sessions = self.sessions.lock().await;
        sessions.insert(id, addr);
    }

    pub async fn remove_session(&self, id: Uuid) {
        let mut sessions = self.sessions.lock().await;
        sessions.remove(&id);
    }

    pub async fn update_progress(&self, id: Uuid, progress: f32) {
        let sessions = self.sessions.lock().await;
        if let Some(socket) = sessions.get(&id) {
            let message = format!("{{\"id\": \"{}\", \"progress\": {}, \"status\": \"In Progress\"}}", id, progress);
            let _ = socket.do_send(WsMessage(message));
        }
    }

    pub async fn complete_process(&self, id: Uuid) {
        let sessions = self.sessions.lock().await;
        if let Some(socket) = sessions.get(&id) {
            let message = format!("{{\"id\": \"{}\", \"progress\": 100, \"status\": \"Completed\"}}", id);
            let _ = socket.do_send(WsMessage(message));
        }
    }
}

impl Actor for FileProcessor {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddSession {
    pub id: Uuid,
    pub addr: Socket,
}

impl Handler<AddSession> for FileProcessor {
    type Result = ();

    fn handle(&mut self, msg: AddSession, _: &mut Context<Self>) {
        let id = msg.id;
        let addr = msg.addr;
        let processor = self.clone();

        actix::spawn(async move {
            processor.add_session(id, addr).await;
        });
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RemoveSession {
    pub id: Uuid,
}

impl Handler<RemoveSession> for FileProcessor {
    type Result = ();

    fn handle(&mut self, msg: RemoveSession, _: &mut Context<Self>) {
        let id = msg.id;
        let processor = self.clone();

        actix::spawn(async move {
            processor.remove_session(id).await;
        });
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateProgress {
    pub id: Uuid,
    pub progress: f32,
}

impl Handler<UpdateProgress> for FileProcessor {
    type Result = ();

    fn handle(&mut self, msg: UpdateProgress, _: &mut Context<Self>) {
        let id = msg.id;
        let progress = msg.progress;
        let processor = self.clone();

        actix::spawn(async move {
            processor.update_progress(id, progress).await;
        });
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CompleteProcess {
    pub id: Uuid,
}

impl Handler<CompleteProcess> for FileProcessor {
    type Result = ();

    fn handle(&mut self, msg: CompleteProcess, _: &mut Context<Self>) {
        let id = msg.id;
        let processor = self.clone();

        actix::spawn(async move {
            processor.complete_process(id).await;
        });
    }
}
