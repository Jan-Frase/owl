use crate::simplified_eval::evaluate_relative;
use mouse::State;
use mouse::backend::constants::STARTING_POS;
use mouse::moove::Moove;
use std::str::SplitWhitespace;
use std::time::Duration;

const INF: i32 = 1_000_000;
const MATE_SCORE: i32 = 100_000;

pub struct Engine {
    pub state: State,
    best_move: Option<Moove>,
    stats: SearchStats,
}

struct SearchStats {
    nodes: u64,
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
            stats: SearchStats { nodes: 0 },
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
            let eval = self.search(depth, 0, -INF, INF);

            // Print an info string to the gui.
            println!(
                "info depth {} nodes {} score {}",
                depth, self.stats.nodes, eval
            );

            // Return if we have no more time...
            let time_elapsed = time_beginning.elapsed();
            if time_limit <= time_elapsed {
                break;
            }

            let search_took = time_search.elapsed();
            let time_left = time_limit - time_elapsed;
            // ... or if this iteration * 15 took longer than the time left,
            // as that usually means that the next iteration will not complete in time.
            if (search_took * 15) > time_left {
                break;
            }
        }

        self.best_move.unwrap()
    }

    // Recursive search function.
    // Implements :
    // - minimax in negation max form
    // - alpha-beta pruning

    // TODO: Next steps:
    // - draw by repetition and 50 move rule
    fn search(&mut self, depth_remaining: i32, sply: i32, mut alpha: i32, beta: i32) -> i32 {
        // Evaluate the position of all leaf nodes.
        // TODO: Quiescence search.
        if depth_remaining <= 0 {
            return evaluate_relative(&self.state);
        };

        // Gen all legal moves.
        let moves = self.state.gen_moves();

        // If we can't move, it is either a stalemate or a checkmate.
        if moves.is_empty() {
            return if self.state.is_in_check() {
                -MATE_SCORE + sply
            } else {
                0
            };
        }

        let mut max_score = -INF;
        for mve in moves {
            self.stats.nodes += 1;

            // TODO: prolly better to implement unmake move eh.
            let old_state = self.state.clone();
            self.state = self.state.make_move(mve);
            // Step deeper into the search.
            let score = -self.search(depth_remaining - 1, sply + 1, -beta, -alpha);
            self.state = old_state;

            // Update the alpha-beta values and max_score.
            if score > max_score {
                max_score = score;
                // Only keep the best move if it is the first move of the search.
                if sply == 0 {
                    self.best_move = Some(mve);
                }
            }

            // If the score is better than the alpha value, update it.
            if score > alpha {
                alpha = score;
            }

            // Alpha-beta cutoff.
            if score >= beta {
                return max_score;
            }
        }

        max_score
    }
}
