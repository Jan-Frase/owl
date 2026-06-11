// This is a simple eval function that I will hopefully replace with a NNUE.
// Based on: https://www.chessprogramming.org/Simplified_Evaluation_Function

use mouse::backend::constants::SQUARES_AMOUNT;
use mouse::bitboard::BitBoard;
use mouse::piece::{Side, ALL_PIECES, ALL_SIDES};
use mouse::piece::Piece::{Pawn, Bishop, Knight, Queen, Rook, King};
use mouse::State;


// Piece Values:
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 10000;

// Piece-Square-Tables:
// Do note that these tables are for black and need to be mirrored for white.
const PAWN_TABLE: [i8; SQUARES_AMOUNT] =
    [
        0,  0,  0,  0,  0,  0,  0,  0,
        50, 50, 50, 50, 50, 50, 50, 50,
        10, 10, 20, 30, 30, 20, 10, 10,
        5,  5, 10, 25, 25, 10,  5,  5,
        0,  0,  0, 20, 20,  0,  0,  0,
        5, -5,-10,  0,  0,-10, -5,  5,
        5, 10, 10,-20,-20, 10, 10,  5,
        0,  0,  0,  0,  0,  0,  0,  0,
    ];

const KNIGHTS_TABLE: [i8; SQUARES_AMOUNT] =
    [
        -50,-40,-30,-30,-30,-30,-40,-50,
        -40,-20,  0,  0,  0,  0,-20,-40,
        -30,  0, 10, 15, 15, 10,  0,-30,
        -30,  5, 15, 20, 20, 15,  5,-30,
        -30,  0, 15, 20, 20, 15,  0,-30,
        -30,  5, 10, 15, 15, 10,  5,-30,
        -40,-20,  0,  5,  5,  0,-20,-40,
        -50,-40,-30,-30,-30,-30,-40,-50,
    ];

const BISHOPS_TABLE: [i8; SQUARES_AMOUNT] =
    [
        -20,-10,-10,-10,-10,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5, 10, 10,  5,  0,-10,
        -10,  5,  5, 10, 10,  5,  5,-10,
        -10,  0, 10, 10, 10, 10,  0,-10,
        -10, 10, 10, 10, 10, 10, 10,-10,
        -10,  5,  0,  0,  0,  0,  5,-10,
        -20,-10,-10,-10,-10,-10,-10,-20,
    ];

const ROOKS_TABLE: [i8; SQUARES_AMOUNT] =
    [
         0,  0,  0,  0,  0,  0,  0,  0,
         5, 10, 10, 10, 10, 10, 10,  5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
         0,  0,  0,  5,  5,  0,  0,  0
    ];

const QUEEN_TABLE: [i8; SQUARES_AMOUNT] =
    [
        -20,-10,-10, -5, -5,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5,  5,  5,  5,  0,-10,
         -5,  0,  5,  5,  5,  5,  0, -5,
          0,  0,  5,  5,  5,  5,  0, -5,
        -10,  5,  5,  5,  5,  5,  0,-10,
        -10,  0,  5,  0,  0,  0,  0,-10,
        -20,-10,-10, -5, -5,-10,-10,-20
    ];

const KING_TABLE_MID: [i8; SQUARES_AMOUNT] =
    [
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -20,-30,-30,-40,-40,-30,-30,-20,
        -10,-20,-20,-20,-20,-20,-20,-10,
        20, 20,  0,  0,  0,  0, 20, 20,
        20, 30, 10,  0,  0, 10, 30, 20
    ];

const KING_TABLE_LATE: [i8; SQUARES_AMOUNT] =
    [
        -50,-40,-30,-20,-20,-30,-40,-50,
        -30,-20,-10,  0,  0,-10,-20,-30,
        -30,-10, 20, 30, 30, 20,-10,-30,
        -30,-10, 30, 40, 40, 30,-10,-30,
        -30,-10, 30, 40, 40, 30,-10,-30,
        -30,-10, 20, 30, 30, 20,-10,-30,
        -30,-30,  0,  0,  0,  0,-30,-30,
        -50,-30,-30,-30,-30,-30,-30,-50
    ];

pub fn evaluate_relative(state: &State) -> i32 {
    let white_eval = evaluate_for_white(state);
    match state.active_side {
        Side::White => white_eval,
        Side::Black => -white_eval,
    }
}

pub fn evaluate_for_white(state: &State) -> i32 {
    let mut eval = 0;

    let bb_mgr = &state.bb_mngr;

    // First, let's figure out if we are in the end-game.
    // This is the case if either of the following is true:
    // 1. Both sides have no queens
    let no_queens = bb_mgr.get_piece_bb(Queen).is_empty();

    // 2. Every side that has a queen has additionally no other major pieces or one minor piece maximum.
    let mut sides_with_queen_have_little_material = true;
    for side in ALL_SIDES {
        let queen_bb = bb_mgr.get_colored_piece_bb(Queen, side);
        if queen_bb.is_empty(){
            continue;
        }
        let rook_bb = bb_mgr.get_colored_piece_bb(Rook, side);
        if !rook_bb.is_empty() {
            sides_with_queen_have_little_material = false;
            break;
        }
        let minor_pieces_amount =
            (bb_mgr.get_colored_piece_bb(Bishop, side) | bb_mgr.get_colored_piece_bb(Knight, side))
                .value.count_ones();
        if minor_pieces_amount > 1 {
            sides_with_queen_have_little_material = false;
            break;
        }
    }

    let is_late_game = no_queens || sides_with_queen_have_little_material;

    // Then, sum up the piece values.
    for piece in ALL_PIECES {
        let piece_value = match piece {
            Pawn => PAWN_VALUE,
            Knight => KNIGHT_VALUE,
            Bishop => BISHOP_VALUE,
            Rook => ROOK_VALUE,
            Queen => QUEEN_VALUE,
            King => KING_VALUE,
        };

        let white_piece_count = bb_mgr.get_colored_piece_bb(piece, Side::White).value.count_ones() as i32;
        let black_piece_count = bb_mgr.get_colored_piece_bb(piece, Side::Black).value.count_ones() as i32;
        eval += white_piece_count * piece_value - black_piece_count * piece_value;
    }

    // Lastly, go over the piece-square-tables.
    for piece in ALL_PIECES {
        let piece_table = match piece {
            Pawn => PAWN_TABLE,
            Knight => KNIGHTS_TABLE,
            Bishop => BISHOPS_TABLE,
            Rook => ROOKS_TABLE,
            Queen => QUEEN_TABLE,
            King => if is_late_game { KING_TABLE_LATE } else { KING_TABLE_MID },
        };

        let white_piece_bb = bb_mgr.get_colored_piece_bb(piece, Side::White);
        let black_piece_bb = bb_mgr.get_colored_piece_bb(piece, Side::Black);

        let mut square_bb  = BitBoard{ value: 1 };
        for square in 0..SQUARES_AMOUNT {
            let white_piece_on_square = (white_piece_bb & square_bb).is_not_empty();
            let black_piece_on_square = (black_piece_bb & square_bb).is_not_empty();
            eval += (white_piece_on_square as i8 * piece_table[square ^ 56] - black_piece_on_square as i8 * piece_table[square]) as i32;
            square_bb <<= 1;
        }
    }

    eval
}

#[test]
fn test_evaluate_relative_01() {
    let state = State::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let eval = evaluate_relative(&state);
    println!("{}", eval);
    assert_eq!(eval, 0);
}

#[test]
fn test_evaluate_relative_02() {
    let state = State::new_from_fen("r1bk1bnr/pppp1ppp/2n5/4p3/4P3/3P4/PPP2PPP/RN1QKBNR w KQ - 0 1");
    let eval = evaluate_relative(&state);
    println!("{}", eval);
    assert!(eval > 0);
}

#[test]
fn test_evaluate_relative_03() {
    let state = State::new_from_fen("rnbqk1nr/ppp2ppp/4p3/3p4/3P4/5N2/PPP1PPPP/RNBK1B1R w - - 0 1");
    let eval = evaluate_relative(&state);
    println!("{}", eval);
    assert!(eval < 0);
}

