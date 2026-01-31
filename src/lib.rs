use std::{collections::HashMap, ops::Deref, str::FromStr};

use omp::{events::Events, main, players::Player, register, types::colour::Colour};

type CommandHandler = fn(context: CommandContext);

struct Command {
    identifier: String,
    handler: Option<CommandHandler>,
    validators: HashMap<u32, ArgValidator>,
}

enum ArgValidator {
    Player,
    String,
    Text,
    Number,
}

enum PlayerValidator {
    Connected,
    Nick(String),
}

impl PlayerValidator {
    fn validate() {}
}

struct ValidatorMessage {
    msg: String,
}

impl From<&str> for ValidatorMessage {
    fn from(s: &str) -> ValidatorMessage {
        ValidatorMessage { msg: s.into() }
    }
}

struct CommandBuilder {
    identifier: String,
    handler: Option<CommandHandler>,
    validators: HashMap<u32, ArgValidator>,
}

impl<'a> CommandBuilder {
    pub fn new(identifier: &str) -> Self {
        Self {
            identifier: identifier.into(),
            validators: HashMap::new(),
            handler: None,
        }
    }

    pub fn validator(mut self, index: u32, validator: ArgValidator) -> Self {
        self.validators.insert(index, validator);

        self
    }

    pub fn handler(mut self, handler: CommandHandler) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn build(self) -> Command {
        Command {
            identifier: self.identifier.to_owned(),
            handler: self.handler,
            validators: self.validators,
        }
    }
}

struct CommandManager {
    commands: Vec<Command>,
}

struct CommandContext<'a> {
    player: Player,
    raw: String,
    arg: CommandArgHandler<'a>,
}

impl<'a> CommandContext<'a> {
    fn new(player: Player, raw: &str, arg: CommandArgHandler<'a>) -> Self {
        Self {
            player,
            raw: raw.into(),
            arg,
        }
    }
}

struct CommandArgHandler<'a> {
    args: Vec<&'a str>,
    index: usize,
}

impl<'a> CommandArgHandler<'a> {
    pub fn next<T: FromStr>(&mut self) -> Option<T> {
        if self.index >= self.args.len() {
            return None;
        }

        let val = self.args.get(self.index)?.parse::<T>().ok();

        if val.is_some() {
            self.index += 1;
        }

        val
    }

    pub fn next_text(&mut self) -> Option<String> {
        if self.index >= self.args.len() {
            return None;
        }

        let text = self.args[self.index..].join(" ");

        self.index = self.args.len();

        Some(text)
    }

    pub fn next_player(&mut self) -> Option<Player> {
        let Some(player_id) = self.next() else {
            return None;
        };

        let Some(player) = Player::from_id(player_id) else {
            return None;
        };

        Some(player)
    }
}

struct MyGM {
    pub command_manager: CommandManager,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn process(&self, player: Player, command_text: String) {
        for command in &self.commands {
            let mut command_split = command_text.split_whitespace();

            let Some(cmd) = command_split
                .next()
                .map(|cmd| cmd.strip_prefix("/"))
                .flatten()
            else {
                return;
            };

            let args: Vec<&str> = command_split.collect();

            if cmd != command.identifier {
                continue;
            }

            let Some(handler) = command.handler else {
                continue;
            };

            handler(CommandContext::new(
                player,
                &command_text,
                CommandArgHandler { args, index: 0 },
            ))
        }
    }

    pub fn add(&mut self, command: Command) {
        self.commands.push(command);
    }
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
    let mut command_manager = CommandManager::new();

    command_manager.add(
        CommandBuilder::new("test")
            .validator(1, ArgValidator::Player)
            .handler(|ctx: CommandContext| {
                ctx.player
                    .send_client_message(Colour::from_rgba(0xffff00ff), "Test.");
            })
            .build(),
    );

    command_manager.add(
        CommandBuilder::new("veh")
            .handler(|mut ctx: CommandContext| {
                // ctx.player
                //     .send_client_message(Colour::from_rgba(0xffff00ff), "Test command");
                //
                // let Some(player) = ctx.arg.next_player() else {
                //     println!("Has no player");
                //     return;
                // };
                //
                // let name = player.get_name();
                //
                // // if let Some(player) = ctx.arg.next_player() {
                // //     println!("Has player {}", player.get_name());
                // // }
                //
                // if let Some(amount) = ctx.arg.next::<i32>() {
                //     println!("Has amount {}", amount);
                // }
                //
                // if let Some(text) = ctx.arg.next_text() {
                //     println!("Text {}", text);
                // }
            })
            .build(),
    );

    let rakstar = MyGM {
        command_manager: command_manager,
    };

    register!(rakstar);
}
