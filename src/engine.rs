use mouse::backend::constants::STARTING_POS;
use mouse::moove::Moove;
use mouse::{State, moves};
use std::str::SplitWhitespace;

pub struct Engine {
    pub state: State,
}

// Various constructors
impl Engine {
    pub fn new() -> Engine {
        let state = State::new_from_fen(STARTING_POS);
        Engine { state }
    }

    pub fn from_state(state: State) -> Engine {
        Engine { state }
    }

    pub fn from_fen(fen: &str) -> Engine {
        let state = State::new_from_fen(&fen);
        Engine { state }
    }

    pub fn from_fen_and_moves(fen: &str, moves: SplitWhitespace) -> Engine {
        let mut state = State::new_from_fen(&fen);

        for move_str in moves {
            let moove = Moove::from(move_str);
            state = state.make_move(moove);
        }

        Engine { state }
    }
}


impl Engine {
    pub fn search_root(&mut self, time_limit: u32) -> Moove {
        let moves = moves(&mut self.state);
        moves[0]
    }
}
