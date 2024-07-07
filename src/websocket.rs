use actix::prelude::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Serialize)]
pub struct OperationStatus {
    pub operation_id: String,
    pub status: String,
    pub progress: f32,
}

#[derive(Message, Serialize)]
#[rtype(result = "()")]
pub struct StatusUpdate(pub OperationStatus);

#[derive(Message)]
#[rtype(String)]
pub struct Connect {
    pub addr: Recipient<StatusUpdate>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
}

pub struct OperationStatusServer {
    sessions: HashMap<String, Recipient<StatusUpdate>>,
    operations: HashMap<String, OperationStatus>,
}

impl OperationStatusServer {
    pub fn new() -> Self {
        OperationStatusServer {
            sessions: HashMap::new(),
            operations: HashMap::new(),
        }
    }

    fn send_status(&self, operation_id: &str) {
        if let Some(status) = self.operations.get(operation_id) {
            if let Some(recipient) = self.sessions.get(operation_id) {
                recipient.do_send(StatusUpdate(status.clone()));
            }
        }
    }
}

impl Actor for OperationStatusServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for OperationStatusServer {
    type Result = String;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = uuid::Uuid::new_v4().to_string();
        self.sessions.insert(id.clone(), msg.addr);
        id
    }
}

impl Handler<Disconnect> for OperationStatusServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
        self.operations.remove(&msg.id);
    }
}

impl Handler<StatusUpdate> for OperationStatusServer {
    type Result = ();

    fn handle(&mut self, msg: StatusUpdate, _: &mut Context<Self>) {
        let operation_id = msg.0.operation_id.clone();
        self.operations.insert(operation_id.clone(), msg.0);
        self.send_status(&operation_id);
    }
}
