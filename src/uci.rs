// UCI Interface implementation based on:
// https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf

use crate::engine::Engine;
use crate::uci::CommandIncoming::*;
use mouse::backend::constants::STARTING_POS;
use mouse::piece::Side;
use std::io;
use std::str::FromStr;
use std::time::Duration;

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
            _ => Unknown(value),
        }
    }
}

pub struct UciInterface {
    engine: Engine,
}

impl UciInterface {
    pub fn new() -> Self {
        UciInterface {
            engine: Engine::from_default_pos(),
        }
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

    fn uci(&self) {
        println!("id name MouseAndOwl");
        println!("id author Jan Frase");
        println!("uciok")
    }

    fn is_ready(&self) {
        println!("readyok")
    }

    fn uci_new_game(&mut self) {
        self.engine = Engine::from_default_pos();
        println!("readyok");
    }

    fn position(&mut self, mut fen_and_moves: &str) {
        if fen_and_moves.trim().eq("startpos") {
            fen_and_moves = STARTING_POS;
        } else {
            fen_and_moves = fen_and_moves.strip_prefix("fen").unwrap();
        }

        let split = fen_and_moves
            .split_once("moves")
            .unwrap_or((fen_and_moves, ""));
        let fen = split.0;
        let moves = split.1;

        self.engine = Engine::from_fen_and_moves(fen, moves.split_whitespace());
    }

    fn go(&mut self, cmd: &str) {
        // Parse options
        // I do not support 'searchmoves', 'ponder', 'depth', 'nodes', 'mate', or 'infinite'.
        // That leaves us with 'wtime', 'btime', 'winc', 'binc' 'movestogo' and 'movetime'.
        let mut wtime: Option<u32> = None;
        let mut btime: Option<u32> = None;
        let mut winc: Option<u32> = None;
        let mut binc: Option<u32> = None;
        let mut movestogo: Option<u32> = None;
        let mut movetime: Option<u32> = None;

        let mut iter = cmd.split_whitespace();
        while let Some(option) = iter.next() {
            let x = iter.next().unwrap();
            let x = u32::from_str(x).unwrap();
            match option {
                "wtime" => wtime = Some(x),
                "btime" => btime = Some(x),
                "winc" => winc = Some(x),
                "binc" => binc = Some(x),
                "movestogo" => movestogo = Some(x),
                "movetime" => movetime = Some(x),
                _ => println!("Error: unsupported option: {}", option),
            }
        }

        let mut time_limit = movetime.unwrap_or(0);

        if movetime.is_none() {
            // Use the basic Time Management formula from:
            // https://www.chessprogramming.org/Time_Management
            // Either using the given movestogo or the default value of 20.
            let movestogo = movestogo.unwrap_or(20);

            let (our_time, our_inc) = match self.engine.state.active_side {
                Side::White => (wtime.unwrap(), winc.unwrap()),
                Side::Black => (btime.unwrap(), binc.unwrap()),
            };

            time_limit = our_time / movestogo + our_inc;
        }
        let time_limit = Duration::from_millis(time_limit as u64);

        // Start search
        let moove = self.engine.search_start(time_limit);
        println!("bestmove {}", moove.to_string())
    }

    fn stop(&mut self) {}

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
