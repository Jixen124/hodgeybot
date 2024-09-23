use std::time::Duration;
use shakmaty::san::{ParseSanError, San, SanError};
use shakmaty::zobrist::{Zobrist64, ZobristHash};
use shakmaty::{Bitboard, Chess, Color, Move, Position};
use shakmaty::uci::{IllegalUciMoveError, UciMove};
use serenity::prelude::*;
use rand::{Rng, thread_rng};
use hodgey_chess_engine::find_best_move_with_time;

pub struct ChessGames;

impl TypeMapKey for ChessGames {
    type Value = Mutex<Vec<ChessGame>>;
}

pub enum MoveError {
    InvalidMove,
    IllegalMove
}

impl From<SanError> for MoveError {
    fn from(_: SanError) -> Self {
        Self::IllegalMove
    }
}

impl From<IllegalUciMoveError> for MoveError {
    fn from(_: IllegalUciMoveError) -> Self {
        Self::IllegalMove
    }
}

impl From<ParseSanError> for MoveError {
    fn from(_: ParseSanError) -> Self {
        Self::InvalidMove
    }
}

#[derive(Clone)]
pub struct ChessGame {
    pub white_id: u64,
    pub black_id: u64,
    chess: Chess,
    pub show_coordinates: bool,
    pub board_flips: bool,
    previously_seen_hashes: Vec<u64>,
}

impl ChessGame {
    pub fn new_game_random_sides(player1_id: u64, player2_id: u64) -> Self {
        if thread_rng().gen_bool(0.5) {
            Self {
                white_id: player1_id,
                black_id: player2_id,
                chess: Chess::default(),
                show_coordinates: true,
                board_flips: false,
                previously_seen_hashes: Vec::new(),
            }
        }
        else {
            Self {
                white_id: player2_id,
                black_id: player1_id,
                chess: Chess::default(),
                show_coordinates: true,
                board_flips: false,
                previously_seen_hashes: Vec::new(),
            }
        }
    }

    pub const fn has_user(&self, id: u64) -> bool {
        self.white_id == id || self.black_id == id
    }

    pub fn id_to_move(&self) -> u64 {
        match self.chess.turn() {
            Color::White => self.white_id,
            Color::Black => self.black_id
        }
    }

    pub fn make_move_unchecked(&mut self, selected_move: Move) {
        self.chess.play_unchecked(&selected_move);
        let new_hash = self.chess.zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal).0;
        self.previously_seen_hashes.push(new_hash);
    }

    pub fn legal_move_from_string(&self, mut move_string: String) -> Result<Move, MoveError> {
        move_string = move_string.replace("o", "O").replace("0", "O");
        
        if let Ok(selected_move) = UciMove::from_ascii(move_string.as_bytes()) {
            let legal_move = selected_move.to_move(&self.chess)?;
            return Ok(legal_move)
        }
        
        let selected_move: San = move_string.parse()?;
        let legal_move = selected_move.to_move(&self.chess)?;
        Ok(legal_move)
    }

    pub fn find_best_move(&mut self) -> Move {
        find_best_move_with_time(&self.chess, Duration::from_secs(1), &mut self.previously_seen_hashes)
    }

    pub fn is_in_check(&self) -> bool {
        self.chess.is_check()
    }

    pub fn to_link(&self) -> String {
        let fen = self.chess.board().board_fen(Bitboard::EMPTY).to_string();
        let mut result = format!("https://www.chess.com/dynboard?fen={fen}&board=bases&piece=classic&size=3");

        if self.show_coordinates {
            result += "&coordinates=1";
        }

        if self.board_flips && self.chess.turn() == Color::Black {
            result += "&flip=1";
        }

        result
    }

    pub fn gameover(&self) -> bool {
        self.chess.is_game_over() || (self.chess.halfmoves() > 100)
    }

    pub fn get_gameover_message(&self) -> &'static str {        
        if self.chess.is_checkmate() {
            "Checkmate!"
        }
        else if self.chess.is_insufficient_material() {
            "Stalemate! Insufficient material."
        }
        else if self.chess.halfmoves() > 100 {
            "Draw by 50 move rule!"
        }
        else {
            "Stalemate!"
        }
    }
}