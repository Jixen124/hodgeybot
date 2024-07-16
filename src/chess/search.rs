use std::cmp::Ordering;
use shakmaty::{zobrist::ZobristHash, Board, Chess, Color, Move, Outcome, Position};

pub fn find_best_move(chess: &Chess) -> Move {
    let moves = chess.legal_moves();
    let mut best_score = f32::NEG_INFINITY;
    let mut best_move = None;
    let color = if chess.turn().is_white() {1} else {-1};

    for m in moves {
        let mut new_chess = chess.clone();
        new_chess.play_unchecked(&m);
        let score = -nega_max(&new_chess, 5, f32::NEG_INFINITY, f32::INFINITY, -color);
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

fn evaluate_position(board: &Board) -> f32 {
    let mut score = 0.0;

    score += 1.0 * board.white().intersect(board.pawns()).count() as f32;
    score += 3.0 * board.white().intersect(board.bishops()).count() as f32;
    score += 3.0 * board.white().intersect(board.knights()).count() as f32;
    score += 5.0 * board.white().intersect(board.rooks()).count() as f32;
    score += 9.0 * board.white().intersect(board.queens()).count() as f32;

    score -= 1.0 * board.black().intersect(board.pawns()).count() as f32;
    score -= 3.0 * board.black().intersect(board.bishops()).count() as f32;
    score -= 3.0 * board.black().intersect(board.knights()).count() as f32;
    score -= 5.0 * board.black().intersect(board.rooks()).count() as f32;
    score -= 9.0 * board.black().intersect(board.queens()).count() as f32;
    
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