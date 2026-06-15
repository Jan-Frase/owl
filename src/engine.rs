use crate::move_list::MoveList;
use crate::simplified_eval::evaluate_relative;
use crate::transposition_table::{TTEntryType, TranspositionTable};
use mouse::State;
use mouse::backend::constants::{A1, STARTING_POS};
use mouse::moove::Moove;
use mouse::piece::Piece::{Bishop, Knight};
use std::str::SplitWhitespace;
use std::time::Duration;

const INF: i32 = 10_000_000;
const MATE_SCORE: i32 = 100_000;
const DRAW_SCORE: i32 = 0;
// Checking time limit every 1024 nodes.
const TIME_OUT_CHECK_FREQUENCY: u64 = 1024;

pub struct Engine {
    pub debug: bool,
    pub state: State,
    tt: TranspositionTable,
    best_move: Option<Moove>,
    stats: SearchStats,
    repetition_stack: Vec<u64>,
    time_limit: Duration,
    start_time: std::time::Instant,
    search_data: SearchData,
}

struct SearchData {
    // Basics:
    is_time_over: bool,
    desired_search_depth: u8,
    best_move: Option<Moove>,
    // For quiescence search logging:
    selective_depth_reached: u8,
}

impl SearchData {
    fn new(desired_search_depth: u8) -> Self {
        Self {
            is_time_over: false,
            desired_search_depth,
            selective_depth_reached: 0,
            best_move: None,
        }
    }
}

impl Default for SearchData {
    fn default() -> Self {
        Self::new(0)
    }
}

struct SearchStats {
    nodes: u64,
    q_nodes: u64,
}

impl Default for SearchStats {
    fn default() -> Self {
        Self {
            nodes: 0,
            q_nodes: 0,
        }
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
            debug: false,
            state,
            tt: TranspositionTable::new(),
            best_move: None,
            stats: SearchStats::default(),
            repetition_stack,
            time_limit: Duration::from_secs(0),
            start_time: std::time::Instant::now(),
            search_data: SearchData::default(),
        }
    }
}

// The core of the engine.
impl Engine {
    pub fn search_start(&mut self, time_limit: Duration) -> Moove {
        self.start_time = std::time::Instant::now();
        self.time_limit = time_limit;
        self.reset_for_new_search();

        // Iterative deepening.
        for depth in 1..=64 {
            self.search_data = SearchData::new(depth);
            // Run the actual search!
            let search_start = std::time::Instant::now();
            let eval = self.search(0, -INF, INF);
            let search_duration = search_start.elapsed();

            // If the search did not properly finish...
            if self.search_data.is_time_over {
                break;
            }

            // Print an info string to the gui.
            println!(
                "info depth {} q-depth {} nodes {} q-nodes {} score {} took {:?}",
                depth,
                self.search_data.selective_depth_reached,
                self.stats.nodes,
                self.stats.q_nodes,
                eval,
                search_duration
            );
            if self.debug {
                println!(
                    "info string repetition-stack-size {}",
                    self.repetition_stack.len()
                );
            }

            // Update the best move if the iteration finished before running out of time.
            self.best_move = self.search_data.best_move;

            // Return if the current depth took more than we have remaining...
            let time_remaining = self.time_remaining();
            match time_remaining {
                None => {
                    break;
                }
                Some(time) => {
                    if search_duration > time {
                        break;
                    }
                }
            }

            // Return if we have no more time...
            if self.is_out_of_time() {
                break;
            }
        }

        if self.debug {
            println!(
                "info string time allocated: {:?}, time taken: {:?}",
                self.time_limit,
                self.start_time.elapsed()
            );
        }

        // If no iteration finished just return a random move...
        self.best_move.unwrap_or(self.state.gen_moves()[0])
    }

    // Recursive search function.
    // Implements:
    // - minimax in negation max form
    // - alpha-beta pruning
    // - draw by repetition, insufficient material and 50 move rule
    // - Time Control after every x nodes
    // - Quiescence search
    // - Basic Move ordering MVV-LVA
    // - TT Table

    // TODO: Next steps:
    // - TT Table improvements,
    // 1. try it for qsearch as well
    // 2. try not storing fail high cases
    // 3. add logging regardings its fullness etc
    // Add nps to info lol
    fn search(&mut self, current_depth: u8, mut alpha: i32, beta: i32) -> i32 {
        if self.depth_out_of_time() {
            return DRAW_SCORE;
        }

        // --- Stalemate detection. ---
        // Check for draw by: repetition, insufficient material and 50 move rule.
        if self.is_drawn() {
            return DRAW_SCORE;
        }

        // --- Transposition Table Pruning ---
        // This is needed later to determine the type of this node.
        let original_alpha = alpha;
        let mut node_type = TTEntryType::Exact;
        // Get TT entry
        let tt_entry = self.tt.get_entry(self.state.zobrist_hash);
        if self.debug && tt_entry.is_some() {
            println!("info string tt hit")
        }
        // and prune if we can.
        let remaining_depth = self.search_data.desired_search_depth - current_depth;
        if self.tt.can_tt_prune(tt_entry, remaining_depth, alpha, beta) {
            return tt_entry.unwrap().score;
        }

        // --- Gen all legal moves. ---
        // If we have a TT entry, use it.
        let tt_move = match tt_entry {
            None => None,
            Some(entry) => Some(entry.recommended_move),
        };
        // The tt_move will always be the first move of the search.
        let move_list = MoveList::new(&self.state, tt_move);

        // --- More Stalemate or Checkmate detection. ---
        // If we can't move, it is either a stalemate or a checkmate.
        if move_list.is_empty() {
            return if self.state.is_in_check() {
                // Mates that are further away are better.
                -MATE_SCORE + current_depth as i32
            } else {
                DRAW_SCORE
            };
        }

        // --- Evaluate the position whenever we reach a leaf node. ---
        if current_depth == self.search_data.desired_search_depth {
            return self.quiescence_search(current_depth, alpha, beta);
        };

        // For TT:
        // The best move for this node!
        let mut best_node_move: Moove = Moove::new(A1, A1);
        // --- Core Search over all moves ---
        let mut max_score = -INF;
        for mve in move_list {
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
                // Always update this for the TT table.
                best_node_move = mve;
                // Only keep the best move if it is the first move of the search.
                if current_depth == 0 {
                    self.search_data.best_move = Some(mve);
                }
            }

            // If the score is better than the alpha value, update it.
            if score > alpha {
                alpha = score;
            }

            // Alpha-beta cutoff.
            if score >= beta {
                node_type = TTEntryType::LowerBound;
                break;
            }
        }

        // --- Update the transposition table ---
        if !self.search_data.is_time_over {
            self.tt.update_tt_table(
                self.state.zobrist_hash,
                max_score,
                best_node_move,
                remaining_depth,
                original_alpha,
                beta,
                node_type,
            );
        }

        max_score
    }

    fn quiescence_search(&mut self, current_depth: u8, mut alpha: i32, beta: i32) -> i32 {
        if self.depth_out_of_time() {
            return DRAW_SCORE;
        }

        // Keep track of the selective depth for the info string.
        self.search_data.selective_depth_reached =
            self.search_data.selective_depth_reached.max(current_depth);

        let attack_list = MoveList::new_only_attacks(&self.state);

        // If there are no more captures, finally evaluate the position.
        if attack_list.is_empty() {
            return evaluate_relative(&self.state);
        }

        // Stand-pat pruning
        // https://www.chessprogramming.org/Quiescence_Search#Standing_Pat
        let mut max_score = evaluate_relative(&self.state);
        if max_score >= beta {
            return max_score;
        }
        if max_score > alpha {
            alpha = max_score;
        }

        // Continue normal q-search.
        for mve in attack_list {
            self.stats.nodes += 1;
            self.stats.q_nodes += 1;
            // TODO: prolly better to implement unmake move eh.
            // Enter the new position and push it onto the repetition stack.
            let old_state = self.state.clone();
            self.state = self.state.make_move(mve);

            // Step deeper into the search.
            let score = -self.quiescence_search(current_depth + 1, -beta, -alpha);
            self.state = old_state;

            // Update the max score.
            if score > max_score {
                max_score = score;
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

// Various helper functions.
impl Engine {
    fn reset_for_new_search(&mut self) {
        self.best_move = None;
        self.stats = SearchStats::default()
    }

    fn is_out_of_time(&self) -> bool {
        self.start_time.elapsed() > self.time_limit
    }

    fn time_remaining(&self) -> Option<Duration> {
        self.time_limit.checked_sub(self.start_time.elapsed())
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

    fn depth_out_of_time(&mut self) -> bool {
        // If we are out of time, return 0.
        if self.search_data.is_time_over {
            return true;
        }
        // Every 1024 nodes, check if we are out of time.
        if self.stats.nodes % TIME_OUT_CHECK_FREQUENCY == 0 && self.is_out_of_time() {
            self.search_data.is_time_over = true;
            return true;
        }
        false
    }
}
