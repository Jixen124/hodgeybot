use std::cmp::Ordering;

use shakmaty::{Chess, Position, Board, Role, Move};

pub fn find_best_move(chess: &Chess) -> Move {
    let moves = chess.legal_moves();
    let mut best_score = f32::NEG_INFINITY;
    let mut best_move = None;
    let color = if chess.turn().is_white() {1} else {-1};

    for m in moves {
        let mut new_chess = chess.clone();
        new_chess.play_unchecked(&m);
        let score = -nega_max(&new_chess, 4, f32::NEG_INFINITY, f32::INFINITY, -color);
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
    //I need to add a test for gameover

    if depth == 0 {
        return evaluate_position(chess.board()) * color as f32;
    }

    let mut max = f32::NEG_INFINITY;

    let mut moves = chess.legal_moves();
    
    moves.sort_unstable_by(|a, b| {
        let a = a.is_capture();
        let b = b.is_capture();
        if a && !b {
            Ordering::Less
        }
        else if b && !a {
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
                    return max;
                }
            }
        }
    }
    
    max
}

fn evaluate_position(board: &Board) -> f32 {
    let mut score = 0.0;
    for (_, piece) in board.clone().into_iter() {
        let multiplier = if piece.color.is_white() {1.0} else {-1.0};
        let role_value = match piece.role {
            Role::Pawn => 1.0,
            Role::Bishop => 3.0,
            Role::Knight => 3.0,
            Role::Rook => 5.0,
            Role::Queen => 9.0,
            Role::King => 0.0
        };

        score += role_value * multiplier;
    }
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