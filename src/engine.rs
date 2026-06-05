use std::str::SplitWhitespace;
use mouse::moove::Moove;
use mouse::{moves, State};
use mouse::backend::constants::STARTING_POS;

pub struct Engine {
    state: State

}

impl Engine {
    pub fn new() -> Engine {
        let state = State::new_from_fen(STARTING_POS);
        Engine {
            state
        }
    }

    pub fn from_state(state: State) -> Engine {
        Engine {
            state
        }
    }

    pub fn from_fen(fen: &str) -> Engine {
        let state = State::new_from_fen(&fen);
        Engine {
            state
        }
    }

    pub fn from_fen_and_moves(fen: &str, mooves: SplitWhitespace) -> Engine {
        let mut state = State::new_from_fen(&fen);

        for move_str in mooves {
            let moove = Moove::from(move_str);
            state = state.make_move(moove);
        }

        Engine {
            state
        }
    }

    pub fn search(&mut self) -> Moove {
        let moves = moves(&mut self.state);
        moves[0]
    }
}