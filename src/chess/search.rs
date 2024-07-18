use std::cmp::Ordering;
use shakmaty::{Board, Chess, Color, Move, Outcome, Position, Role};

const INFINITY: isize = isize::MAX;
const NEG_INFINITY: isize = isize::MIN +1;

pub fn find_best_move(chess: &Chess, depth: usize) -> Move {
    let moves = chess.legal_moves();
    let mut best_score = NEG_INFINITY;
    let mut best_move = None;
    let color = if chess.turn().is_white() {1} else {-1};

    for m in moves {
        let mut new_chess = chess.clone();
        new_chess.play_unchecked(&m);
        let score = -nega_max(&new_chess, depth, NEG_INFINITY, INFINITY, -color);
        if score > best_score {
            best_score = score;
            best_move = Some(m)
        }
    }

    if let Some(best_move) = best_move {
        return best_move;
    }

    panic!("NO BEST MOVE");
}

const fn calculate_move_score(m: &Move) -> isize {
    let mut score = if m.is_promotion() {60} else {0};
    if let Some(role) = m.capture() {
        score += match role {
            Role::Pawn => 10,
            Role::Bishop => 30,
            Role::Knight => 30,
            Role::Rook => 50,
            _ => 90
        }
    }
    score
}

pub fn nega_max(chess: &Chess, depth: usize, mut alpha: isize, beta: isize, color: isize) -> isize {
    //Confirm this works
    if let Some(outcome) = chess.outcome() {
        return match outcome {
            Outcome::Draw => 0,
            Outcome::Decisive { winner } => {
                color * if winner == Color::White {
                    1_000_000 + depth as isize
                }
                else {
                    -1_000_000 - depth as isize
                }
            }
        }
    }

    if depth == 0 {
        return evaluate_position(chess.board()) * color as isize;
    }

    let mut max = NEG_INFINITY;

    let mut moves = chess.legal_moves();
    
    moves.sort_unstable_by(|a, b| {
        // let a_score = calculate_move_score(a);
        // let b_score = calculate_move_score(b);

        // if a_score > b_score {
        //     Ordering::Less
        // }
        // else if a_score < b_score {
        //     Ordering::Greater
        // }
        // else {
        //     Ordering::Equal
        // }

        let a_is_interesting = a.is_capture() || a.is_promotion();
        let b_is_interesting = b.is_capture() || b.is_promotion();

        if a_is_interesting && !b_is_interesting {
            Ordering::Less
        }
        else if b_is_interesting && !a_is_interesting {
            Ordering::Greater
        }
        else {
            Ordering::Equal
        }
    });

    for m in &moves {
        let mut new_chess = chess.clone();
        new_chess.play_unchecked(m);
        let score = -nega_max(&new_chess, depth - 1, -beta, -alpha, -color);
        if score > max {
            max = score;
            if max > alpha {
                alpha = max;
                if alpha >= beta {
                    break;
                }
            }
        }
    }
    
    max
}

//Piece square tables from https://www.chessprogramming.org/Simplified_Evaluation_Function

const PAWN_PCSQ: [isize; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
  150, 150, 150, 150, 150, 150, 150, 150,
  110, 110, 120, 130, 130, 120, 110, 110,
  105, 105, 110, 125, 125, 110, 105, 105,
  100, 100, 100, 120, 120, 100, 100, 100,
  105,  95,  90, 100, 100,  90,  95, 105,
  105, 110, 110,  80,  80, 110, 110, 105,
    0,   0,   0,   0,   0,   0,   0,   0
];

const KNIGHT_PCSQ: [isize; 64] = [
  250, 260, 270, 270, 270, 270, 260, 250,
  260, 280, 300, 300, 300, 300, 280, 260,
  270, 300, 310, 315, 315, 310, 300, 270,
  270, 305, 315, 320, 320, 315, 305, 270,
  270, 300, 315, 320, 320, 315, 300, 270,
  270, 305, 310, 315, 315, 310, 305, 270,
  260, 280, 300, 305, 305, 300, 280, 260,
  250, 260, 270, 270, 270, 270, 260, 250
];

const BISHOP_PCSQ: [isize; 64] = [
  280, 290, 290, 290, 290, 290, 290, 280,
  290, 300, 300, 300, 300, 300, 300, 290,
  290, 300, 305, 310, 310, 305, 300, 290,
  290, 305, 305, 310, 310, 305, 305, 290,
  290, 300, 310, 310, 310, 310, 300, 290,
  290, 310, 310, 310, 310, 310, 310, 290,
  290, 305, 300, 300, 300, 300, 305, 290,
  280, 290, 290, 290, 290, 290, 290, 280
];

const ROOK_PCSQ: [isize; 64] = [
  500, 500, 500, 500, 500, 500, 500, 500,
  505, 510, 510, 510, 510, 510, 510, 505,
  495, 500, 500, 500, 500, 500, 500, 495,
  495, 500, 500, 500, 500, 500, 500, 495,
  495, 500, 500, 500, 500, 500, 500, 495,
  495, 500, 500, 500, 500, 500, 500, 495,
  495, 500, 500, 500, 500, 500, 500, 495,
  500, 500, 500, 505, 505, 500, 500, 500
];


const QUEEN_PCSQ: [isize; 64] = [
  880, 890, 890, 895, 895, 890, 890, 880,
  890, 900, 900, 900, 900, 900, 900, 890,
  890, 900, 905, 905, 905, 905, 900, 890,
  895, 900, 905, 905, 905, 905, 900, 895,
  900, 900, 905, 905, 905, 905, 900, 895,
  890, 905, 905, 905, 905, 905, 900, 890,
  890, 900, 905, 900, 900, 900, 900, 890,
  880, 890, 890, 895, 895, 890, 890, 880
];

const KING_PCSQ: [isize; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20    
];

fn evaluate_position(board: &Board) -> isize {
    let mut score = 0;

    for square in board.white().intersect(board.pawns()) {
        score += PAWN_PCSQ[square as usize];
    }
    for square in board.white().intersect(board.bishops()) {
        score += BISHOP_PCSQ[square as usize];
    }
    for square in board.white().intersect(board.knights()) {
        score += KNIGHT_PCSQ[square as usize];
    }
    for square in board.white().intersect(board.rooks()) {
        score += ROOK_PCSQ[square as usize];
    }
    for square in board.white().intersect(board.queens()) {
        score += QUEEN_PCSQ[square as usize];
    }
    score += KING_PCSQ[board.king_of(Color::White).unwrap() as usize];

    for square in board.black().intersect(board.pawns()) {
        score -= PAWN_PCSQ[square as usize ^ 56];
    }
    for square in board.black().intersect(board.bishops()) {
        score -= BISHOP_PCSQ[square as usize ^ 56];
    }
    for square in board.black().intersect(board.knights()) {
        score -= KNIGHT_PCSQ[square as usize ^ 56];
    }
    for square in board.black().intersect(board.rooks()) {
        score -= ROOK_PCSQ[square as usize ^ 56];
    }
    for square in board.black().intersect(board.queens()) {
        score -= QUEEN_PCSQ[square as usize ^ 56];
    }
    score -= KING_PCSQ[board.king_of(Color::Black).unwrap() as usize ^ 56];
    score
}

// #[cfg(test)]
// mod tests {
//     use shakmaty::fen::Fen;
//     use shakmaty::{CastlingMode, Chess, FromSetup};
//     use super::super::test_fens;

//     #[test]
//     fn test_fens_time() {
//         for fen in test_fens::WIN_AT_CHESS {
//             let setup = Fen::from_ascii(fen.as_bytes()).expect("Fen should be valid").0;
//             let chess = Chess::from_setup(setup, CastlingMode::Standard).expect("position should be valid");
//             super::find_best_move(&chess, 3);
//         }
//     }
// }