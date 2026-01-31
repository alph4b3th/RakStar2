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
        self.text = Some(text.into());
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
