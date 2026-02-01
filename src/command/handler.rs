use std::{collections::HashMap, str::FromStr};

use omp::players::Player;

pub type CommandHandler = fn(context: CommandContext) -> Result<(), String>;

pub struct Command {
    pub identifier: String,
    pub handler: Option<CommandHandler>,
    pub validators: HashMap<u32, ArgValidator>,
    pub subcommands: Vec<Command>,
}

pub enum ArgValidator {
    Player(PlayerConstraints),
    String(StringConstraints),
    Number(NumberConstraints),
    Text,
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
            ArgValidator::Number(constraints) => {
                let number: i32 = arg.parse().map_err(|_| "Invalid number".to_string())?;
                constraints.validate(number)
            }
            ArgValidator::String(constraints) => constraints.validate(arg),
            ArgValidator::Text => Ok(()),
        }
    }
}

pub struct NumberConstraints {
    min: Option<i32>,
    max: Option<i32>,
    positive: bool,
}

impl NumberConstraints {
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
            positive: false,
        }
    }

    fn validate(&self, value: i32) -> Result<(), String> {
        if self.positive && value <= 0 {
            return Err("Number must be positive".to_string());
        }

        if let Some(min) = self.min {
            if value < min {
                return Err(format!("Number must be at least {}", min));
            }
        }

        if let Some(max) = self.max {
            if value > max {
                return Err(format!("Number must be at most {}", max));
            }
        }

        Ok(())
    }

    pub fn min(mut self, value: i32) -> Self {
        self.min = Some(value);
        self
    }

    pub fn max(mut self, value: i32) -> Self {
        self.max = Some(value);
        self
    }

    pub fn positive(mut self) -> Self {
        self.positive = true;
        self
    }
}

pub struct StringConstraints {
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<String>,
}

impl StringConstraints {
    pub fn new() -> Self {
        Self {
            min_length: None,
            max_length: None,
            pattern: None,
        }
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        if let Some(min) = self.min_length {
            if value.len() < min {
                return Err(format!("String must be at least {} characters", min));
            }
        }

        if let Some(max) = self.max_length {
            if value.len() > max {
                return Err(format!("String must be at most {} characters", max));
            }
        }

        if let Some(ref pattern) = self.pattern {
            if !value.contains(pattern) {
                return Err(format!("String must contain '{}'", pattern));
            }
        }

        Ok(())
    }

    pub fn min_length(mut self, length: usize) -> Self {
        self.min_length = Some(length);
        self
    }

    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);
        self
    }

    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.into());
        self
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
    subcommands: Vec<Command>,
}

impl<'a> CommandBuilder {
    pub fn new(identifier: &str) -> Self {
        Self {
            identifier: identifier.into(),
            validators: HashMap::new(),
            handler: None,
            subcommands: Vec::new(),
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

    pub fn subcommand(mut self, subcommand: Command) -> Self {
        self.subcommands.push(subcommand);
        self
    }

    pub fn build(self) -> Command {
        Command {
            identifier: self.identifier.to_owned(),
            handler: self.handler,
            validators: self.validators,
            subcommands: self.subcommands,
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
    fn check_validator_type(
        &self,
        expected: &str,
        matcher: impl Fn(&ArgValidator) -> bool,
    ) -> Result<(), String> {
        if let Some(validator) = self.validators.get(&(self.index as u32)) {
            if !matcher(validator) {
                let actual_type = match validator {
                    ArgValidator::Player(_) => "Player",
                    ArgValidator::Number(_) => "Number",
                    ArgValidator::String(_) => "String",
                    ArgValidator::Text => "Text",
                };
                return Err(format!(
                    "Validator mismatch at position {}: expected {}, but you used next_{}()",
                    self.index,
                    actual_type,
                    expected.to_lowercase()
                ));
            }
        }
        Ok(())
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
        self.check_validator_type("Player", |v| matches!(v, ArgValidator::Player(_)))?;

        if self.index >= self.args.len() {
            return Err("Missing player argument".to_string());
        }

        let arg = self.args[self.index];
        let player_id: i32 = arg.parse().map_err(|_| "Invalid player id".to_string())?;
        self.index += 1;

        let player =
            Player::from_id(player_id).ok_or_else(|| format!("Player {} not found", player_id))?;

        Ok(player)
    }

    pub fn next_number<T: FromStr>(&mut self) -> Result<T, String> {
        self.check_validator_type("Number", |v| matches!(v, ArgValidator::Number(_)))?;

        if self.index >= self.args.len() {
            return Err("Missing number argument".to_string());
        }

        let arg = self.args[self.index];
        let val = arg.parse::<T>().map_err(|_| "Invalid number".to_string())?;
        self.index += 1;

        Ok(val)
    }

    pub fn next_string(&mut self) -> Result<String, String> {
        self.check_validator_type("String", |v| matches!(v, ArgValidator::String(_)))?;

        if self.index >= self.args.len() {
            return Err("Missing string argument".to_string());
        }

        let arg = self.args[self.index].to_string();
        self.index += 1;

        Ok(arg)
    }
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn process(&self, player: Player, command_text: String) {
        let mut command_split = command_text.split_whitespace();

        let Some(cmd) = command_split
            .next()
            .map(|cmd| cmd.strip_prefix("/"))
            .flatten()
        else {
            return;
        };

        for command in &self.commands {
            if cmd != command.identifier {
                continue;
            }

            let args: Vec<&str> = command_split.collect();
            self.process_command(command, &args, player, &command_text);
            return;
        }
    }

    fn process_command(
        &self,
        command: &Command,
        args: &[&str],
        player: Player,
        full_command_text: &str,
    ) {
        if let Some(handler) = command.handler {
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

            let result = handler(CommandContext::new(
                player,
                full_command_text,
                CommandArgHandler {
                    args: args.to_vec(),
                    index: 0,
                    validators: &command.validators,
                },
            ));

            if let Err(msg) = result {
                player.send_client_message(omp::types::colour::Colour::from_rgba(0xFF0000FF), &msg);
                return;
            }

            if command.subcommands.is_empty() {
                return;
            }
        }

        if !args.is_empty() && !command.subcommands.is_empty() {
            for subcommand in &command.subcommands {
                if subcommand.identifier == args[0] {
                    self.process_command(subcommand, &args[1..], player, full_command_text);
                    return;
                }
            }
        }
    }

    pub fn add(&mut self, command: Command) {
        self.commands.push(command);
    }
}
