use crate::simplified_eval::get_piece_value;
use mouse::State;
use mouse::moove::Moove;
use mouse::piece::Piece::Pawn;

const HASH_MOVE_BONUS: i16 = 30000;
const CAPTURE_BONUS: i16 = 10000;

pub struct MoveList {
    moves: Vec<Moove>,
    scores: Vec<i16>,
}

impl MoveList {
    pub fn new(state: &State, tt_move: Option<Moove>) -> MoveList {
        let moves = state.gen_moves();
        let scores = Self::score_moves(state, &moves, tt_move);

        MoveList { moves, scores }
    }

    pub fn new_only_attacks(state: &State) -> MoveList {
        let moves = state.gen_attacks();
        let scores = Self::score_moves(state, &moves, None);

        MoveList { moves, scores }
    }

    fn score_moves(state: &State, moves: &Vec<Moove>, tt_move: Option<Moove>) -> Vec<i16> {
        let mut scores = Vec::with_capacity(moves.len());

        for mve in moves {
            scores.push(Self::score_move(state, mve, tt_move));
        }

        scores
    }

    // According to: https://www.chessprogramming.org/Move_Ordering
    // The ordering of scoring usually goes something like this:
    // 1. 30_000 Hash Move: Is this the best move from a prior search at this position?
    // 2. 10_000 - 20_000 MVV-LVA: Most Valuable Victim - Least Valuable Aggressor
    // TODO: 3. Promotions
    // TODO: 4 and 5. Killer moves?
    // 6. TODO: History - no idea yet
    fn score_move(state: &State, mve: &Moove, tt_move: Option<Moove>) -> i16 {
        // If we have a hash move, and this is it, ensure that it's at the front!
        if let Some(tt_move) = tt_move
            && *mve == tt_move
        {
            return HASH_MOVE_BONUS;
        }

        // Get attacker and defender.
        // It's a bit convoluted because of en passant.
        let attacker = state.bb_mngr.get_piece_at_square(mve.get_from()).unwrap();
        let defender = match state.irreversible_data.en_passant_square {
            None => state.bb_mngr.get_piece_at_square(mve.get_to()),
            Some(ep_square) => {
                if attacker == Pawn && mve.get_to() == ep_square {
                    Some(Pawn)
                } else {
                    None
                }
            }
        };

        // MVV - LVA
        if let Some(defender) = defender {
            let attacker_value = get_piece_value(attacker) as i16;
            let defender_value = get_piece_value(defender) as i16;
            return CAPTURE_BONUS + defender_value - attacker_value;
        }

        0
    }

    fn selection_sort_step(&mut self) {
        let mut best_score = self.scores[0];
        let mut best_index = 0;

        for i in 1..self.scores.len() {
            let score = self.scores[i];
            if score > best_score {
                best_index = i;
                best_score = score;
            }
        }

        if self.scores.len() != self.moves.len() {
            panic!("scores and moves are not the same length");
        }
        let end = self.scores.len() - 1;
        self.scores.swap(end, best_index);
        self.moves.swap(end, best_index);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }
}

impl Iterator for MoveList {
    type Item = Moove;

    fn next(&mut self) -> Option<Self::Item> {
        // Runs a single iteration of the selection sort.
        // This shifts the best move to the end of the list.
        if !self.moves.is_empty() {
            self.selection_sort_step();
        }
        // Which we can then pop here :)
        self.scores.pop();
        self.moves.pop()
    }
}
