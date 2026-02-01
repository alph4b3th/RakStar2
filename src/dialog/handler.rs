use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;


#[derive(Debug, Clone)]
pub struct DialogResponse {
    pub dialog_id: u32,
    pub player_id: u32,
    pub button_response: u8,
    pub selected_item: i8,
    pub answer: String,
}


struct DialogWorker {
    sender: Option<oneshot::Sender<DialogResponse>>,
}

impl DialogWorker {
    fn new() -> (Self, oneshot::Receiver<DialogResponse>) {
        let (tx, rx) = oneshot::channel();
        (Self { sender: Some(tx) }, rx)
    }

    fn notify(&mut self, resp: DialogResponse) {
        if let Some(tx) = self.sender.take() {
            let _ = tx.send(resp);
        }
    }
}


struct DialogManager {
    counter: u32,
    workers: HashMap<u32, DialogWorker>, 
}

impl DialogManager {
    fn new() -> Self {
        Self {
            counter: 0,
            workers: HashMap::new(),
        }
    }

    fn send(&mut self) -> (u32, oneshot::Receiver<DialogResponse>) {
        self.counter += 1;
        let dialog_id = self.counter;

        let (worker, rx) = DialogWorker::new();
        self.workers.insert(dialog_id, worker);

        (dialog_id, rx)
    }

  
    fn notify(
        &mut self,
        dialog_id: u32,
        player_id: u32,
        button_response: u8,
        selected_item: i8,
        answer: String,
    ) {
        if let Some(mut worker) = self.workers.remove(&dialog_id) {
            let resp = DialogResponse {
                dialog_id,
                player_id,
                button_response,
                selected_item,
                answer,
            };
            worker.notify(resp);
        }
    }
}


static DIALOG_MANAGER: Lazy<Arc<Mutex<DialogManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(DialogManager::new())));

pub fn notify_dialog(
    dialog_id: u32,
    player_id: u32,
    button_response: u8,
    selected_item: i8,
    answer: String,
) {
    let mut manager = DIALOG_MANAGER.lock().unwrap();
    manager.notify(dialog_id, player_id, button_response, selected_item, answer);
}


pub struct DialogBuilder {
    title: String,
    message: String,
}

impl DialogBuilder {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            message: String::new(),
        }
    }

    pub fn set_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn set_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }


    pub fn send_dialog(self) -> oneshot::Receiver<DialogResponse> {
        let mut manager = DIALOG_MANAGER.lock().unwrap();
        let (_dialog_id, rx) = manager.send();
        rx
    }
}
