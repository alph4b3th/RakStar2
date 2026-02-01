use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use once_cell::sync::Lazy;

#[derive(Debug)]
pub struct DialogResponse {
    pub dialog_id: u32,
    pub answer: String,
}

struct DialogWorker {
    sender: Option<oneshot::Sender<DialogResponse>>,
}

impl DialogWorker {
    pub fn new() -> (Self, oneshot::Receiver<DialogResponse>) {
        let (tx, rx) = oneshot::channel();
        (Self { sender: Some(tx) }, rx)
    }

    pub fn notify(&mut self, dialog_id: u32, answer: String) {
        if let Some(tx) = self.sender.take() {
            let _ = tx.send(DialogResponse { dialog_id, answer });
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

    fn send(&mut self) -> u32 {
        self.counter += 1;
        let dialog_id = self.counter;

        let (worker, rx) = DialogWorker::new();
        self.workers.insert(dialog_id, worker);

        // Spawn da task aguardadora
        tokio::spawn(async move {
            match rx.await {
                Ok(resp) => println!(
                    "Dialog {} recebeu resposta: {}",
                    resp.dialog_id, resp.answer
                ),
                Err(_) => println!("Dialog {} foi cancelada", dialog_id),
            }
        });

        dialog_id
    }

    fn notify(&mut self, dialog_id: u32, answer: String) {
        if let Some(mut worker) = self.workers.remove(&dialog_id) {
            worker.notify(dialog_id, answer);
        }
    }
}


static DIALOG_MANAGER: Lazy<Arc<Mutex<DialogManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(DialogManager::new())));


pub fn send_dialog() -> u32 {
    let mut manager = DIALOG_MANAGER.lock().unwrap();
    manager.send()
}

pub fn notify_dialog(dialog_id: u32, answer: String) {
    let mut manager = DIALOG_MANAGER.lock().unwrap();
    manager.notify(dialog_id, answer)
}