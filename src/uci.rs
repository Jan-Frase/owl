// UCI Interface implementation based on:
// https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf

use std::io;
use mouse::backend::constants::STARTING_POS;
use crate::engine::Engine;
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
    Perft(String),
    Unknown(String), // We have recieved some invalid or unsupported command.
}

impl From<String> for CommandIncoming {
    fn from(value: String) -> Self {
        let split_value = value.split_once(" ").unwrap_or((value.as_str(), ""));
        let command = split_value.0;
        let options = String::from(split_value.1);
        match command.trim() {
            "uci" => Uci,
            "isready" => IsReady,
            "ucinewgame" => UciNewGame,
            "position" => Position(options),
            "go" => Go(options),
            "stop" => Stop,
            "quit" => Quit,
            _ => Unknown(value)
        }
    }
}

pub struct UciInterface {
    engine: Engine,
}

impl UciInterface {
    pub fn new() -> Self {
        UciInterface{ engine: Engine::new() }
    }

    pub fn run(&mut self) {
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
                UciNewGame => self.uci_new_game(),
                Position(fen_and_moves) => self.position(fen_and_moves.as_str()),
                Go(cmd) => self.go(cmd.as_str()),
                Stop => self.stop(),
                Quit => break,
                Perft(cmd) => self.perft(cmd.as_str()),
                Unknown(line) => println!("Unknown: {}", line),
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

    fn uci_new_game(&mut self) {
        self.engine = Engine::new();
        println!("readyok");
    }

    fn position(&mut self, mut fen_and_moves: &str) {
        if fen_and_moves.eq("startpos") {
            fen_and_moves = STARTING_POS;
        } else {
            fen_and_moves = fen_and_moves.strip_prefix("fen").unwrap();
        }

        let split = fen_and_moves.split_once("moves").unwrap_or((fen_and_moves, ""));
        let fen = split.0;
        let moves = split.1;

        self.engine = Engine::from_fen_and_moves(fen, moves.split_whitespace());
    }

    fn go(&mut self, cmd: &str) {
        let moove = self.engine.search();
        println!("bestmove {}", moove.to_string())
    }

    fn stop(&mut self) {

    }

    fn perft(&mut self, cmd: &str) {
        let depth: i32 = match cmd.parse::<i32>() {
            Ok(value) => value,
            Err(_) => {
                println!("Error: depth must be an integer.");
                return;
            }
        };
    }
}
