use crate::utils;
use omp::types::colour::Colour;

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
        // let bytes = utils::encode::cp_1252::to_cp1252(text);
        // let msg = utils::encode::cp_1252::cp1252_bytes_to_str(&bytes);
        let msg = text;
      
        self.text = Some(msg.into());
        self
    }

    pub fn select(mut self, player_id: i32) -> Self {
        self.player_id = Some(player_id);
        self
    }

    pub fn send(self) -> Self {
        let Some(player_id) = self.player_id else {
            return self.send_all();
        };

        let Some(player) = omp::players::Player::from_id(player_id) else {
            println!("jogador: {} off.", player_id);
            return self;
        };

        let Some(text) = self.text.as_deref() else {
            return self;
        };

        player.send_client_message(Colour::from_rgba(0xFFFFFFFF), &text);
        println!("message enviada: {:?}", text);

        return self;
    }

    pub fn send_range(self) -> Self {
        let Some(player_id) = self.player_id else {
            return self.send_all();
        };

        let Some(player) = omp::players::Player::from_id(player_id) else {
            println!("jogador: {} off.", 0);
            return self;
        };

        let player_pos = player.get_pos();
        let player_name = player.get_name();

        for id in 0..omp::core::MaxPlayers() {
            let Some(player) = omp::players::Player::from_id(id) else {
                println!("jogador: {} off.", 0);
                continue;
            };

            let target_pos = player.get_pos();
            let distance = utils::distance_point::point::calculate(
                player_pos.x,
                player_pos.y,
                player_pos.z,
                target_pos.x,
                target_pos.y,
                target_pos.z,
            );

            if (distance < 30.0) {
                let Some(text) = self.text.as_deref() else {
                    continue;
                };

                let message = format!("{} disse: {}", player_name, text);
                
                player.send_client_message(Colour::from_rgba(0xFFFFFFFF), &message);
            }
        }

        return self;
    }

    pub fn send_all(self) -> Self {
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
