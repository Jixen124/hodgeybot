use shakmaty::san::{ParseSanError, San, SanError};
use shakmaty::{Bitboard, Chess, Color, Move, Position};
use shakmaty::uci::{IllegalUciMoveError, UciMove};
use serenity::prelude::*;
use rand::{Rng, thread_rng};

mod search;
mod test_fens;

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
            }
        }
        else {
            Self {
                white_id: player2_id,
                black_id: player1_id,
                chess: Chess::default(),
                show_coordinates: true,
                board_flips: false,
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
    }

    pub fn legal_move_from_str(&self, move_str: &str) -> Result<Move, MoveError> {
        if let Ok(selected_move) = UciMove::from_ascii(move_str.as_bytes()) {
            let legal_move = selected_move.to_move(&self.chess)?;
            return Ok(legal_move)
        }
        
        let selected_move: San = move_str.parse()?;
        let legal_move = selected_move.to_move(&self.chess)?;
        Ok(legal_move)
    }

    pub fn find_best_move(&self) -> Move {
        search::find_best_move(&self.chess, 6)
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
        self.chess.is_game_over()
    }

    pub fn get_gameover_message(&self) -> &'static str {        
        if self.chess.is_checkmate() {
            "Checkmate!"
        }
        else if self.chess.is_insufficient_material() {
            "Stalemate! Insufficient material."
        }
        else {
            "Stalemate!"
        }
    }
}