// UCI Interface implementation based on:
// https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf

use std::io;
use std::io::Write;
use crate::uci::CommandIncoming::*;

// All commands that the GUI might send the engine.
// Currently, commands that are not supported are commented out.
enum CommandIncoming {
    Uci,
    // Debug,
    IsReady,
    // SetOption,
    // Register,
    UciNewGame,
    Position(String),
    Go(String),
    Stop,
    // PonderHit,
    Quit,
    Unknown, // We have recieved some invalid or unsupported command.
}

impl From<String> for CommandIncoming {
    fn from(value: String) -> Self {
        match value.trim() {
            "uci" => Uci,
            "isready" => IsReady,
            "ucinewgame" => UciNewGame,
            "position" => Position(value.strip_prefix("position").unwrap_or("").to_string()),
            "go" => Go(value.strip_prefix("go").unwrap_or("").to_string()),
            "stop" => Stop,
            "quit" => Quit,
            _ => Unknown
        }
    }
}

// All commands that the engine might send the GUI.
// Currently, commands that are not supported are commented out.
enum ComandOutgoing {
    Id,
    UciOk,
    ReadyOk,
    BestMove,
    // CopyProtection,
    // Registration,
    Info,
    // Option,
}

pub struct UciInterface {}

impl UciInterface {
    pub fn new() -> Self {
        UciInterface{}
    }

    pub fn run(&self,) {
        loop {
            // This blocks until input arrives.
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();

            // Parse to command.
            let command = CommandIncoming::from(line);

            // Execute the command.
            match command {
                Uci => self.uci(),
                IsReady => self.is_ready(),
                UciNewGame => {}
                Position(fen_and_moves) => {}
                Go(cmd) => {}
                Stop => {}
                Quit => {}
                Unknown => {}
            }
        }
    }

    fn uci(&self, ) {
        println!("id name MouseAndOwl");
        println!("id author Jan Frase");
        println!("uciok")
    }

    fn is_ready(&self) {
        println!("readyok")
    }
}
