use mouse::backend::constants::STARTING_POS;
use mouse::moove::Moove;
use mouse::{State, moves};
use std::str::SplitWhitespace;
use std::time::Duration;
use crate::simplified_eval::evaluate_relative;

pub struct Engine {
    pub state: State,
    best_move: Option<Moove>,
    stats: SearchStats,
}

struct SearchStats {
    nodes: i32,
}

impl Default for SearchStats {
    fn default() -> Self {
        Self { nodes: 0 }
    }
}

// Various constructors and some house-keeping.
impl Engine {
    pub fn from_default_pos() -> Engine {
        let state = State::new_from_fen(STARTING_POS);
        Self::new(state)
    }

    pub fn new(state: State) -> Engine {
        Engine {
            state,
            best_move: None,
            stats: SearchStats { nodes: 0 }
        }
    }


    pub fn from_fen_and_moves(fen: &str, moves: SplitWhitespace) -> Engine {
        let mut state = State::new_from_fen(&fen);

        for move_str in moves {
            let moove = Moove::from(move_str);
            state = state.make_move(moove);
        }

        Self::new(state)
    }

    fn reset_for_new_search(&mut self) {
        self.best_move = None;
        self.stats = SearchStats::default()
    }
}


// The core of the engine.
impl Engine {
    pub fn search_start(&mut self, time_limit: Duration) -> Moove {
        let time_beginning = std::time::Instant::now();
        self.reset_for_new_search();

        // Iterative deepening.
        for depth in 1..=64 {
            let time_search = std::time::Instant::now();
            // Run the actual search!
            let eval = self.search(depth, 0);
            // Print an info string to the gui.
            println!("info depth {} nodes {} score {}", depth, self.stats.nodes, eval);

            // Return if we have no more time...
            let time_elapsed = time_beginning.elapsed();
            if time_limit <= time_elapsed { break; }

            let search_took = time_search.elapsed();
            let time_left = time_limit - time_elapsed;
            // ... or if this iteration * 15 took longer than the time left,
            // as that usually means that the next iteration will not complete in time.
            if (search_took * 15) > time_left { break; }
        }

        self.best_move.unwrap()
    }

    // Recursive search function.
    // Implements :
    // - minimax
    // TODO: Next steps:
    // - testing via crucible
    // - draw by repetition and 50 move rule
    // - alpha-beta pruning
    fn search(&mut self, depth: i32, sply: i32) -> i32 {
        if depth == 0 { return evaluate_relative(&self.state) };

        let mut max_score = i32::MIN;
        let moves = self.state.gen_moves();

        // If we can't move, it is either a stalemate or a checkmate.
        if moves.is_empty() {
            return if self.state.is_in_check() { i32::MAX } else { 0 }
        }

        for mve in moves {
            self.stats.nodes += 1;
            // TODO: prolly better to implement unmake move eh.
            let old_state = self.state.clone();
            self.state = self.state.make_move(mve);
            let score = -self.search(depth - 1, sply + 1);
            self.state = old_state;
            if score > max_score {
                max_score = score;
                if sply == 0 {
                    self.best_move = Some(mve);
                }
            }
        }

        max_score
    }
}
