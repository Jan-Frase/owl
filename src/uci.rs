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
    Debug(String),
    IsReady,
    SetOption(String),
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
            "debug" => Debug(options),
            "isready" => IsReady,
            "setoption" => SetOption(options),
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
                Debug(options) => self.debug(options.as_str()),
                IsReady => self.is_ready(),
                SetOption(options) => self.set_option(options.as_str()),
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
        println!("option name Logging type check");
        println!("uciok")
    }

    fn debug(&mut self, options: &str) {
        match options {
            "on" => self.engine.debug = true,
            "off" => self.engine.debug = false,
            _ => println!("info string Unsupported debug option: {}", options),
        }
    }

    fn is_ready(&self) {
        println!("readyok")
    }

    fn set_option(&mut self, option: &str) {
        let mut iter = option.split_whitespace();
        let name = iter.next().unwrap();
        if name != "name" {
            println!("info string Unsupported option string: {}", option);
            return;
        }
        let name = iter.next().unwrap();
        let value = iter.next().unwrap();
        if value != "value" {
            println!("info string Unsupported option string: {}", option);
            return;
        }
        let value = iter.next().unwrap();
        match name {
            "Logging" => {
                self.engine.debug = value == "true";
            }
            _ => println!("info string Unsupported option: {}", option),
        }
    }

    fn uci_new_game(&mut self) {
        self.engine = Engine::from_default_pos();
        println!("readyok");
    }

    fn position(&mut self, fen_and_moves: &str) {
        let mut iter = fen_and_moves.split_whitespace();

        let next = iter.next().unwrap();
        let fen = match next {
            "startpos" => String::from(STARTING_POS),
            // fen is all strings concatenated until we hit `moves` or the string ends.
            "fen" => iter.by_ref().take(6).collect::<Vec<&str>>().join(" "),
            _ => {
                println!("info string Unsupported fen: {}", next);
                panic!("Unsupported fen");
            }
        };

        // Skip the 'moves' word if it exists.
        iter.next();

        self.engine = Engine::from_fen_and_moves(fen.as_str(), iter);
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
                _ => println!("info string Unsupported option: {}", option),
            }
        }

        let mut time_limit = movetime.unwrap_or(0);

        if movetime.is_none() {
            // Use the basic Time Management formula from:
            // https://www.chessprogramming.org/Time_Management
            // Either using the given movestogo or the default value of 25.
            let movestogo = movestogo.unwrap_or(25);

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
            Err(err) => {
                println!("info string Unsupported depth {}.", err);
                return;
            }
        };
    }
}
