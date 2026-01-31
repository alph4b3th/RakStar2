use std::{collections::HashMap, ops::Deref, str::FromStr};

mod chat;
mod command;
use chat::*;
use command::*;
use omp::{events::Events, main, register, types::colour::Colour};

struct MyGM {
    command_manager: CommandManager,
}

impl Events for MyGM {
    fn on_player_connect(&mut self, player: omp::players::Player) {
        player.send_client_message(Colour::from_rgba(0xFFFFFFFF), "Welcome to my server!");
    }

    fn on_player_command_text(&mut self, player: omp::players::Player, message: String) -> bool {
        self.command_manager.process(player, message);

        true
    }
}

#[main]
pub fn game_main() {
    let command_manager = CommandManager::new();

    let rakstar = MyGM {
        command_manager: command_manager,
    };

    chat::oi();

    register!(rakstar);
}
