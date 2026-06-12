use crate::simplified_eval::evaluate_relative;
use mouse::State;
use mouse::backend::constants::STARTING_POS;
use mouse::moove::Moove;
use mouse::piece::Piece::{Bishop, Knight};
use std::str::SplitWhitespace;
use std::time::Duration;

const INF: i32 = 1_000_000;
const MATE_SCORE: i32 = 100_000;
const DRAW_SCORE: i32 = 0;

pub struct Engine {
    pub state: State,
    best_move: Option<Moove>,
    desired_search_depth: i32,
    stats: SearchStats,
    repetition_stack: Vec<u64>,
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
        Self::from_fen_and_moves(STARTING_POS, "".split_whitespace())
    }

    pub fn from_fen_and_moves(fen: &str, moves: SplitWhitespace) -> Engine {
        let mut repetition_stack = Vec::with_capacity(100);

        let mut state = State::new_from_fen(&fen);
        repetition_stack.push(state.zobrist_hash);

        for move_str in moves {
            let moove = Moove::from(move_str);
            state = state.make_move(moove);
            if state.half_move_clock == 0 {
                repetition_stack.clear();
            }
            repetition_stack.push(state.zobrist_hash);
        }

        Engine {
            state,
            best_move: None,
            desired_search_depth: 0,
            stats: SearchStats::default(),
            repetition_stack,
        }
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
            self.desired_search_depth = depth;
            let time_search = std::time::Instant::now();
            // Run the actual search!
            let eval = self.search(0, -INF, INF);

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
    // Implements:
    // - minimax in negation max form
    // - alpha-beta pruning

    // TODO: Next steps:
    // - draw by repetition, insufficient material and 50 move rule
    // - Quiescence search.
    fn search(&mut self, current_depth: i32, mut alpha: i32, beta: i32) -> i32 {
        // Check for draw by: repetition, insufficient material and 50 move rule.
        if self.is_drawn() {
            return DRAW_SCORE;
        }

        // Gen all legal moves.
        let moves = self.state.gen_moves();

        // If we can't move, it is either a stalemate or a checkmate.
        if moves.is_empty() {
            return if self.state.is_in_check() {
                // Mates that are further away are better.
                -MATE_SCORE + current_depth
            } else {
                DRAW_SCORE
            };
        }

        // --- If it isn't already over, evaluate the position whenever we reach a leaf node. ---
        if current_depth == self.desired_search_depth {
            return evaluate_relative(&self.state);
        };

        let mut max_score = -INF;
        for mve in moves {
            self.stats.nodes += 1;

            // TODO: prolly better to implement unmake move eh.
            // Enter the new position and push it onto the repetition stack.
            let old_state = self.state.clone();
            self.state = self.state.make_move(mve);

            // Step deeper into the search.
            self.repetition_stack.push(self.state.zobrist_hash);
            let score = -self.search(current_depth + 1, -beta, -alpha);
            self.repetition_stack.pop();
            self.state = old_state;

            // Update the alpha-beta values and max_score.
            if score > max_score {
                max_score = score;
                // Only keep the best move if it is the first move of the search.
                if current_depth == 0 {
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

    fn is_drawn(&self) -> bool {
        // --- Check for the 50-move-rule. ---
        if self.state.half_move_clock >= 100 {
            return true;
        }

        // --- Check for draw by repetition. ---
        // -1 to adjust for len vs. index,
        let mut index: i32 = self.repetition_stack.len() as i32 - 1;
        let start_index = index - self.state.half_move_clock as i32;
        // -2 because we are interested in our previous position
        index -= 2;
        let mut repetition_counter = 0;
        loop {
            if index < start_index || index < 0 {
                break;
            }
            if self.repetition_stack[index as usize] == self.state.zobrist_hash {
                repetition_counter += 1;
                if repetition_counter == 2 {
                    return true;
                }
            }
            if index < 2 {
                break;
            }
            index -= 2;
        }

        // --- Check for (at least some cases of) insufficient material. ---
        // There are more cases, but we don't attempt to catch them.
        // Are there only three pieces left?
        if self.state.bb_mngr.get_occupied_bb().value.count_ones() == 3 {
            // Is one of those pieces a bishop or knight?
            let bishop_or_knight_bb =
                self.state.bb_mngr.get_piece_bb(Bishop) | self.state.bb_mngr.get_piece_bb(Knight);
            if bishop_or_knight_bb.is_not_empty() {
                return true;
            }
        }

        false
    }
}
