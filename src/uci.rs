// UCI Interface implementation based on:
// https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf

use std::cmp::min;
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
    // Perft(String),
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

        self.engine.set_position(fen.as_str(), iter);
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

        let (soft_limit, hard_limit) = self.calc_time_allotment(movetime, wtime, winc, btime, binc);

        // Start search
        let moove = self.engine.search_start(soft_limit, hard_limit);
        println!("bestmove {}", moove.to_string())
    }

    fn calc_time_allotment(&self, move_time: Option<u32>, wtime: Option<u32>, winc: Option<u32>, btime: Option<u32>, binc: Option<u32>) -> (Duration, Duration) {
        if let Some(move_time) = move_time {
            let limit = Duration::from_millis(move_time as u64);
            return (limit, limit);
        }

        // Use the basic Time Management formula from:
        // https://www.chessprogramming.org/Time_Management
        let (our_time, our_inc) = match self.engine.state.active_side {
            Side::White => (wtime.unwrap(), winc.unwrap()),
            Side::Black => (btime.unwrap(), binc.unwrap()),
        };

        let our_time: f64 = our_time as f64;
        let our_inc: f64 = our_inc as f64;
        let movestogo: f64 = 80.0;

        let soft_limit = our_time / movestogo + our_inc ;
        let hard_limit = (soft_limit * 2.0).min(our_time * 0.5 + our_inc);
        let soft_limit = Duration::from_millis(soft_limit as u64);
        let hard_limit = Duration::from_millis(hard_limit as u64);

        (soft_limit, hard_limit)
    }

    fn stop(&mut self) {}
}
