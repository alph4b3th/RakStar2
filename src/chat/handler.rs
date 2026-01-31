use omp::types::colour::Colour;
use crate::utils;

pub struct MsgBuilder {
    pub text: Option<String>,
    player_id: Option<i32>,
}

impl MsgBuilder {
    pub fn new() -> Self {
        Self {
            text: None,
            player_id: None,
        }
    }

    pub fn text(mut self, text: &str) -> Self {
        let bytes = utils::encode::cp_1252::to_cp1252(text);
        let msg = utils::encode::cp_1252::cp1252_bytes_to_str(&bytes);
        self.text =  Some(msg.into());
        
        self
    }

    pub fn send(mut self) -> Self {
        for id in 0..omp::core::MaxPlayers() {
            let Some(player) = omp::players::Player::from_id(id) else {
                println!("jogador: {} off.", id);
                continue;
            };

            let Some(text) = self.text.as_deref() else {
                continue;
            };

            player.send_client_message(Colour::from_rgba(0xFFFFFFFF), &text);
            println!("message enviada: {:?}", text);
        }

        self
    }


}

pub fn oi() {}
