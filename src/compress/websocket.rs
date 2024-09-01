use actix::*;
use crate::ws::*;

use serde_json::json;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use futures::lock::Mutex;

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

    pub async fn update_progress(&self, id: Uuid, files: Vec<FileProgress>, status: Status) {
        let sessions = self.sessions.lock().await;
        if let Some(socket) = sessions.get(&id) {
            let message = json!(
                {
                    "id": id.to_string(),
                    "files": files,
                    "status": status
                }
            ).to_string();
            socket.do_send(WsMessage(message));
        }
    }

    pub async fn complete_process(&self, id: Uuid, files: Vec<FileProgress>,status: Status) {
        let sessions = self.sessions.lock().await;
        if let Some(socket) = sessions.get(&id) {
            let message = json!(
                {
                    "id": id.to_string(),
                    "files": files,
                    "status": status,
                }
            ).to_string();
            socket.do_send(WsMessage(message));
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
    pub status: Status,
    pub files: Vec<FileProgress>,
}

impl Handler<UpdateProgress> for FileProcessor {
    type Result = ();

    fn handle(&mut self, msg: UpdateProgress, _: &mut Context<Self>) {
        let id = msg.id;
        // let progress = msg.progress;
        let status = msg.status;
        // let message = msg.message;
        let files = msg.files;
        let processor = self.clone();

        actix::spawn(async move {
            processor.update_progress(id, files, status).await;
        });
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CompleteProcess {
    pub id: Uuid,
    pub status: Status,
    pub files: Vec<FileProgress>,
}

impl Handler<CompleteProcess> for FileProcessor {
    type Result = ();

    fn handle(&mut self, msg: CompleteProcess, _: &mut Context<Self>) {
        let id = msg.id;
        let status = msg.status;
        let files = msg.files;
        let processor = self.clone();

        actix::spawn(async move {
            processor.complete_process(id, files, status).await;
        });
    }
}
