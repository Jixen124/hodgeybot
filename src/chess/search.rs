use std::cmp::Ordering;
use shakmaty::{Board, Chess, Color, Move, Outcome, Position};

const INFINITY: isize = isize::MAX;
const NEG_INFINITY: isize = isize::MIN +1;

pub fn find_best_move(chess: &Chess) -> Move {
    let moves = chess.legal_moves();
    let mut best_score = NEG_INFINITY;
    let mut best_move = None;
    let color = if chess.turn().is_white() {1} else {-1};

    for m in moves {
        let mut new_chess = chess.clone();
        new_chess.play_unchecked(&m);
        let score = -nega_max(&new_chess, 6, NEG_INFINITY, INFINITY, -color);
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

pub fn nega_max(chess: &Chess, depth: usize, mut alpha: isize, beta: isize, color: isize) -> isize {
    //Confirm this works
    if let Some(outcome) = chess.outcome() {
        return match outcome {
            Outcome::Draw => 0,
            Outcome::Decisive { winner } => {
                if winner == Color::White && color == 1 || winner == Color::Black && color == -1 {
                    INFINITY
                }
                else {
                    NEG_INFINITY
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

const PAWN_PCSQ: [isize; 64] = [
    0,   0,  0,    0,   0,   0,   0,   0,
  105, 110, 115, 120, 120, 115, 110, 105,
  104, 108, 112, 116, 116, 112, 108, 104,
  103, 106, 109, 112, 112, 109, 106, 103,
  102, 104, 106, 108, 108, 106, 104, 102,
  101, 102, 103,  90,  90,   3, 102, 101,
  100, 100, 100,  60,  60, 100, 100, 100,
    0,   0,   0,   0,   0,   0,   0,   0
];

const KNIGHT_PCSQ: [isize; 64] = [
  290, 290, 290, 290, 290, 290, 290, 290,
  290, 300, 300, 300, 300, 300, 300, 290,
  290, 300, 305, 305, 305, 305, 300, 290,
  290, 300, 305, 310, 310, 305, 300, 290,
  290, 300, 305, 310, 310, 305, 300, 290,
  290, 300, 305, 305, 305, 305, 300, 290,
  290, 300, 300, 300, 300, 300, 300, 290,
  290, 270, 290, 290, 290, 290, 270, 290
];

const BISHOP_PCSQ: [isize; 64] = [
  290, 290, 290, 290, 290, 290, 290, 290,
  290, 300, 300, 300, 300, 300, 300, 290,
  290, 300, 305, 305, 305, 305, 300, 290,
  290, 300, 305, 310, 310, 305, 300, 290,
  290, 300, 305, 310, 310, 305, 300, 290,
  290, 300, 305, 305, 305, 305, 300, 290,
  290, 300, 300, 300, 300, 300, 300, 290,
  290, 290, 280, 290, 290, 280, 290, 290
];

const KING_PCSQ: [isize; 64] = [
  -40, -40, -40, -40, -40, -40, -40, -40,
  -40, -40, -40, -40, -40, -40, -40, -40,
  -40, -40, -40, -40, -40, -40, -40, -40,
  -40, -40, -40, -40, -40, -40, -40, -40,
  -40, -40, -40, -40, -40, -40, -40, -40,
  -40, -40, -40, -40, -40, -40, -40, -40,
  -20, -20, -20, -20, -20, -20, -20, -20,
    0,  20,  40, -20,   0, -20,  40,  20
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

    //score += 100 * board.white().intersect(board.pawns()).count() as isize;
    // score += 300 * board.white().intersect(board.bishops()).count() as isize;
    // score += 300 * board.white().intersect(board.knights()).count() as isize;
    score += 500 * board.white().intersect(board.rooks()).count() as isize;
    score += 900 * board.white().intersect(board.queens()).count() as isize;

    // score -= 100 * board.black().intersect(board.pawns()).count() as isize;
    // score -= 300 * board.black().intersect(board.bishops()).count() as isize;
    // score -= 300 * board.black().intersect(board.knights()).count() as isize;
    score -= 500 * board.black().intersect(board.rooks()).count() as isize;
    score -= 900 * board.black().intersect(board.queens()).count() as isize;
    
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