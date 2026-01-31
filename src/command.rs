use std::{collections::HashMap, str::FromStr};

use omp::players::Player;

pub type CommandHandler = fn(context: CommandContext);

pub struct Command {
    pub identifier: String,
    pub handler: Option<CommandHandler>,
    pub validators: HashMap<u32, ArgValidator>,
}

pub enum ArgValidator {
    Player(PlayerConstraints),
    String,
    Text,
    Number,
}

impl ArgValidator {
    fn validate(&self, arg: &str) -> Result<(), String> {
        match self {
            ArgValidator::Player(constraints) => {
                let id: i32 = arg.parse().map_err(|_| "Invalid player ID".to_string())?;

                let player =
                    Player::from_id(id).ok_or_else(|| format!("Player {} not found", id))?;

                constraints.validate(&player)
            }
            ArgValidator::Number => Ok(()),
            ArgValidator::String => Ok(()),
            ArgValidator::Text => Ok(()),
        }
    }
}

pub struct PlayerConstraints {
    min_health: Option<f32>,
    max_health: Option<f32>,
    spawned: bool,
    nick: Option<String>,
}

impl PlayerConstraints {
    pub fn new() -> Self {
        Self {
            min_health: None,
            max_health: None,
            nick: None,
            spawned: false,
        }
    }

    fn validate(&self, player: &Player) -> Result<(), String> {
        if self.spawned && !player.is_spawned() {
            return Err("Player must be spawned".to_string());
        }

        if let Some(ref nick) = self.nick {
            if !player.get_name().contains(nick) {
                return Err(format!("Player name must contain '{}'", nick));
            }
        }

        if let Some(min) = self.min_health {
            if player.get_health() < min {
                return Err(format!("Player health must be at least {}", min));
            }
        }

        if let Some(max) = self.max_health {
            if player.get_health() > max {
                return Err(format!("Player health must be at most {}", max));
            }
        }

        Ok(())
    }

    pub fn min_health(mut self, health: f32) -> Self {
        self.min_health = Some(health);
        self
    }

    pub fn max_health(mut self, health: f32) -> Self {
        self.max_health = Some(health);
        self
    }

    pub fn must_be_spawned(mut self) -> Self {
        self.spawned = true;
        self
    }

    pub fn with_nick(mut self, nick: &str) -> Self {
        self.nick = Some(nick.into());
        self
    }
}

struct ValidatorMessage {
    msg: String,
}

impl From<&str> for ValidatorMessage {
    fn from(s: &str) -> ValidatorMessage {
        ValidatorMessage { msg: s.into() }
    }
}

pub struct CommandBuilder {
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

pub struct CommandManager {
    commands: Vec<Command>,
}

pub struct CommandContext<'a> {
    pub player: Player,
    pub raw: String,
    pub arg: CommandArgHandler<'a>,
}

impl<'a> CommandContext<'a> {
    pub fn new(player: Player, raw: &str, arg: CommandArgHandler<'a>) -> Self {
        Self {
            player,
            raw: raw.into(),
            arg,
        }
    }
}

pub struct CommandArgHandler<'a> {
    pub args: Vec<&'a str>,
    pub index: usize,
    pub validators: &'a HashMap<u32, ArgValidator>,
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

    pub fn next_player(&mut self) -> Result<Player, String> {
        if let Some(validator) = self.validators.get(&(self.index as u32)) {
            if !matches!(validator, ArgValidator::Player(_)) {
                return Err(format!(
                    "Validator type mismatch: expected Player validator at position {}",
                    self.index
                ));
            }
        }

        let Some(player_id) = self.next() else {
            return Err("Invalid player id.".into());
        };

        let Some(player) = Player::from_id(player_id) else {
            return Err(String::from("Invalid player"));
        };

        Ok(player)
    }
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

            for (index, validator) in &command.validators {
                let Some(arg) = args.get(*index as usize) else {
                    player.send_client_message(
                        omp::types::colour::Colour::from_rgba(0xFF0000FF),
                        &format!("Missing argument at position {}", index),
                    );
                    return;
                };

                if let Err(msg) = validator.validate(arg) {
                    player.send_client_message(
                        omp::types::colour::Colour::from_rgba(0xFF0000FF),
                        &msg,
                    );
                    return;
                }
            }

            let Some(handler) = command.handler else {
                continue;
            };

            handler(CommandContext::new(
                player,
                &command_text,
                CommandArgHandler {
                    args,
                    index: 0,
                    validators: &command.validators,
                },
            ))
        }
    }

    pub fn add(&mut self, command: Command) {
        self.commands.push(command);
    }
}
