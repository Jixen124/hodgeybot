use std::cmp::Ordering;
use shakmaty::{Board, Chess, Color, Move, Outcome, Position};

pub fn find_best_move(chess: &Chess) -> Move {
    let moves = chess.legal_moves();
    let mut best_score = f32::NEG_INFINITY;
    let mut best_move = None;
    let color = if chess.turn().is_white() {1} else {-1};

    for m in moves {
        let mut new_chess = chess.clone();
        new_chess.play_unchecked(&m);
        let score = -nega_max(&new_chess, 6, f32::NEG_INFINITY, f32::INFINITY, -color);
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

pub fn nega_max(chess: &Chess, depth: usize, mut alpha: f32, beta: f32, color: i32) -> f32 {
    //Confirm this works
    if let Some(outcome) = chess.outcome() {
        return match outcome {
            Outcome::Draw => 0.0,
            Outcome::Decisive { winner } => {
                if winner == Color::White && color == 1 || winner == Color::Black && color == -1 {
                    f32::INFINITY
                }
                else {
                    f32::NEG_INFINITY
                }
            }
        }
    }

    if depth == 0 {
        return evaluate_position(chess.board()) * color as f32;
    }

    let mut max = f32::NEG_INFINITY;

    let mut moves = chess.legal_moves();
    
    moves.sort_unstable_by(|a, b| {
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

//Piece square tables from https://github.com/terredeciels/TSCP/blob/master/eval.c

const PAWN_PCSQ: [f32; 64] = [
    0.0,   0.0,  0.0,    0.0,   0.0,   0.0,   0.0,   0.0,
  105.0, 110.0, 115.0, 120.0, 120.0, 115.0, 110.0, 105.0,
  104.0, 108.0, 112.0, 116.0, 116.0, 112.0, 108.0, 104.0,
  103.0, 106.0, 109.0, 112.0, 112.0, 109.0, 106.0, 103.0,
  102.0, 104.0, 106.0, 108.0, 108.0, 106.0, 104.0, 102.0,
  101.0, 102.0, 103.0,  90.0,  90.0,   3.0, 102.0, 101.0,
  100.0, 100.0, 100.0,  60.0,  60.0, 100.0, 100.0, 100.0,
    0.0,   0.0,   0.0,   0.0,   0.0,   0.0,   0.0,   0.0
];

const KNIGHT_PCSQ: [f32; 64] = [
  290.0, 290.0, 290.0, 290.0, 290.0, 290.0, 290.0, 290.0,
  290.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 290.0,
  290.0, 300.0, 305.0, 305.0, 305.0, 305.0, 300.0, 290.0,
  290.0, 300.0, 305.0, 310.0, 310.0, 305.0, 300.0, 290.0,
  290.0, 300.0, 305.0, 310.0, 310.0, 305.0, 300.0, 290.0,
  290.0, 300.0, 305.0, 305.0, 305.0, 305.0, 300.0, 290.0,
  290.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 290.0,
  290.0, 270.0, 290.0, 290.0, 290.0, 290.0, 270.0, 290.0
];

const BISHOP_PCSQ: [f32; 64] = [
  290.0, 290.0, 290.0, 290.0, 290.0, 290.0, 290.0, 290.0,
  290.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 290.0,
  290.0, 300.0, 305.0, 305.0, 305.0, 305.0, 300.0, 290.0,
  290.0, 300.0, 305.0, 310.0, 310.0, 305.0, 300.0, 290.0,
  290.0, 300.0, 305.0, 310.0, 310.0, 305.0, 300.0, 290.0,
  290.0, 300.0, 305.0, 305.0, 305.0, 305.0, 300.0, 290.0,
  290.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 290.0,
  290.0, 290.0, 280.0, 290.0, 290.0, 280.0, 290.0, 290.0
];

const KING_PCSQ: [f32; 64] = [
  -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0,
  -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0,
  -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0,
  -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0,
  -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0,
  -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0, -40.0,
  -20.0, -20.0, -20.0, -20.0, -20.0, -20.0, -20.0, -20.0,
    0.0,  20.0,  40.0, -20.0,   0.0, -20.0,  40.0,  20.0
];

const FLIP: [usize; 64] = [
    56,  57,  58,  59,  60,  61,  62,  63,
    48,  49,  50,  51,  52,  53,  54,  55,
    40,  41,  42,  43,  44,  45,  46,  47,
    32,  33,  34,  35,  36,  37,  38,  39,
    24,  25,  26,  27,  28,  29,  30,  31,
    16,  17,  18,  19,  20,  21,  22,  23,
     8,   9,  10,  11,  12,  13,  14,  15,
     0,   1,   2,   3,   4,   5,   6,   7
];

fn evaluate_position(board: &Board) -> f32 {
    let mut score = 0.0;

    for square in board.white().intersect(board.pawns()) {
        score += PAWN_PCSQ[square as usize];
    }
    for square in board.white().intersect(board.bishops()) {
        score += BISHOP_PCSQ[square as usize];
    }
    for square in board.white().intersect(board.knights()) {
        score += KNIGHT_PCSQ[square as usize];
    }
    score += KING_PCSQ[board.king_of(Color::White).unwrap() as usize];

    for square in board.black().intersect(board.pawns()) {
        score -= PAWN_PCSQ[FLIP[square as usize]];
    }
    for square in board.black().intersect(board.bishops()) {
        score -= BISHOP_PCSQ[FLIP[square as usize]];
    }
    for square in board.black().intersect(board.knights()) {
        score -= KNIGHT_PCSQ[FLIP[square as usize]];
    }
    score += KING_PCSQ[FLIP[board.king_of(Color::Black).unwrap() as usize]];

    //score += 100.0 * board.white().intersect(board.pawns()).count() as f32;
    // score += 300.0 * board.white().intersect(board.bishops()).count() as f32;
    // score += 300.0 * board.white().intersect(board.knights()).count() as f32;
    score += 500.0 * board.white().intersect(board.rooks()).count() as f32;
    score += 900.0 * board.white().intersect(board.queens()).count() as f32;

    // score -= 100.0 * board.black().intersect(board.pawns()).count() as f32;
    // score -= 300.0 * board.black().intersect(board.bishops()).count() as f32;
    // score -= 300.0 * board.black().intersect(board.knights()).count() as f32;
    score -= 500.0 * board.black().intersect(board.rooks()).count() as f32;
    score -= 900.0 * board.black().intersect(board.queens()).count() as f32;
    
    score
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use shakmaty::Chess;

    #[test]
    fn time() {
        let start_time = Instant::now();
        super::find_best_move(&Chess::new());
        println!("Time: {}", Instant::now().duration_since(start_time).as_millis());
    }
}