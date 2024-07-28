use shakmaty::{zobrist::{Zobrist64, ZobristHash}, Board, Chess, Color, Move, Outcome, Position, Role};

mod piece_square_tables;
mod test_fens;

const INFINITY: i16 = i16::MAX;
const NEG_INFINITY: i16 = -INFINITY;
const TRANSPOSITION_TABLE_SIZE: usize = 1024 * 1024 * 8;
const TABLE_INDEX_MASK: usize = TRANSPOSITION_TABLE_SIZE - 1;

#[derive(Clone, Copy, PartialEq)]
enum TranspositionTableFlag {
    None,
    Exact,
    Lowerbound,
    Upperbound
}

#[derive(Clone, Copy)]
struct TranspositionTableData {
    hash: u64,
    score: i16,
    depth: u16,
    flag: TranspositionTableFlag
}

impl TranspositionTableData {
    fn empty() -> TranspositionTableData {
        TranspositionTableData {
            hash: 0,
            score: 0,
            depth: 0,
            flag: TranspositionTableFlag::None
        }
    }
}

pub fn find_best_move(chess: &Chess, max_depth: u16) -> Move {
    let mut moves = chess.legal_moves();
    moves.sort_unstable_by_key(|m| move_score(m));
    let mut best_score = NEG_INFINITY;
    let mut best_move = None;

    let mut transposition_table: Vec<TranspositionTableData> = vec![TranspositionTableData::empty(); TRANSPOSITION_TABLE_SIZE];
    for m in moves {
        let mut new_chess = chess.clone();
        new_chess.play_unchecked(&m);

        let score = -nega_max(&new_chess, max_depth, NEG_INFINITY, INFINITY, &mut transposition_table);
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

fn nega_max(chess: &Chess, depth: u16, mut alpha: i16, mut beta: i16, transposition_table: &mut Vec<TranspositionTableData>) -> i16 {
    if let Some(outcome) = chess.outcome() {
        return match outcome {
            Outcome::Draw => 0,
            _ => -32_000 - depth as i16
        };
    }

    if depth == 0 {
        return evaluate_position(chess.board()) * if chess.turn().is_white() {1} else {-1};
    }
    
    let original_alpha = alpha;

    let hash: Zobrist64 = chess.zobrist_hash(shakmaty::EnPassantMode::Legal);
    let table_index = hash.0 as usize & TABLE_INDEX_MASK;
    if transposition_table[table_index].hash == hash.0 && transposition_table[table_index].depth >= depth {
        if transposition_table[table_index].flag == TranspositionTableFlag::Exact {
            return transposition_table[table_index].score;
        }
        else if transposition_table[table_index].flag == TranspositionTableFlag::Lowerbound {
            alpha = alpha.max(transposition_table[table_index].score);
        }
        else if transposition_table[table_index].flag == TranspositionTableFlag::Upperbound {
            beta = beta.min(transposition_table[table_index].score);
        }
        
        if alpha >= beta {
            return transposition_table[table_index].score;
        }
    }

    let mut value = NEG_INFINITY;

    let mut moves = chess.legal_moves();
    moves.sort_unstable_by_key(|m| move_score(m));

    for m in &moves {
        let mut new_chess = chess.clone();
        new_chess.play_unchecked(m);
        let score = -nega_max(&new_chess, depth - 1, -beta, -alpha, transposition_table);
        value = value.max(score);
        alpha = alpha.max(value);
        if alpha >= beta {
            break;
        }
    }

    if transposition_table[table_index].depth < depth {
        transposition_table[table_index].hash = hash.0;
        transposition_table[table_index].score = value;
        transposition_table[table_index].depth = depth;

        transposition_table[table_index].flag = if value <= original_alpha {
            TranspositionTableFlag::Upperbound
        }
        else if value >= beta {
            TranspositionTableFlag::Lowerbound
        }
        else {
            TranspositionTableFlag::Exact
        }
    }
    
    value
}

const fn move_score(m: &Move) -> i16 {
    let mut score = if m.is_promotion() {60} else {0};
    if let Some(role) = m.capture() {
        score += match m.role() {
            Role::Pawn => 1,
            Role::Bishop => 3,
            Role::Knight => 3,
            Role::Rook => 5,
            _ => 9
        } - match role {
            Role::Pawn => 10,
            Role::Bishop => 30,
            Role::Knight => 30,
            Role::Rook => 50,
            _ => 90
        }
    }
    score
}

fn evaluate_position(board: &Board) -> i16 {
    let mut score = 0;

    for square in board.white().intersect(board.pawns()) {
        score += piece_square_tables::PAWN[square as usize];
    }
    for square in board.white().intersect(board.bishops()) {
        score += piece_square_tables::BISHOP[square as usize];
    }
    for square in board.white().intersect(board.knights()) {
        score += piece_square_tables::KNIGHT[square as usize];
    }
    for square in board.white().intersect(board.rooks()) {
        score += piece_square_tables::ROOK[square as usize];
    }
    for square in board.white().intersect(board.queens()) {
        score += piece_square_tables::QUEEN[square as usize];
    }
    score += piece_square_tables::KING[board.king_of(Color::White).unwrap() as usize];

    for square in board.black().intersect(board.pawns()) {
        score -= piece_square_tables::PAWN[square as usize ^ 56];
    }
    for square in board.black().intersect(board.bishops()) {
        score -= piece_square_tables::BISHOP[square as usize ^ 56];
    }
    for square in board.black().intersect(board.knights()) {
        score -= piece_square_tables::KNIGHT[square as usize ^ 56];
    }
    for square in board.black().intersect(board.rooks()) {
        score -= piece_square_tables::ROOK[square as usize ^ 56];
    }
    for square in board.black().intersect(board.queens()) {
        score -= piece_square_tables::QUEEN[square as usize ^ 56];
    }
    score -= piece_square_tables::KING[board.king_of(Color::Black).unwrap() as usize ^ 56];
    score
}

#[cfg(test)]
mod tests {
    use shakmaty::fen::Fen;
    use shakmaty::{CastlingMode, Chess, FromSetup};
    use super::test_fens;

    #[test]
    fn test_fens_time() {
        for fen in test_fens::WIN_AT_CHESS {
            let setup = Fen::from_ascii(fen.as_bytes()).expect("Fen should be valid").0;
            let chess = Chess::from_setup(setup, CastlingMode::Standard).expect("position should be valid");
            super::find_best_move(&chess, 2);
        }
    }

    #[test]
    fn test_position_time() {
        let setup = Fen::from_ascii("2rq1bk1/1b4pp/pn3n2/1p1Ppp2/1PP1P3/7P/3N1PP1/R2QRBK1 w - - 0 23".as_bytes()).expect("Fen should be valid").0;
        let chess = Chess::from_setup(setup, CastlingMode::Standard).expect("position should be valid");
        super::find_best_move(&chess, 6);
    }

    #[test]
    fn lasker_position() {
        let setup = Fen::from_ascii("8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - -".as_bytes()).expect("Fen should be valid").0;
        let chess = Chess::from_setup(setup, CastlingMode::Standard).expect("position should be valid");
        assert!(super::find_best_move(&chess, 20).to_string() == "Ka1-b1");
    }
}