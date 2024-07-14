use std::str::FromStr;
use chess::{Board, ChessMove, Color, MoveGen};
use serenity::prelude::*;
use rand::{Rng, thread_rng, seq::IteratorRandom};

pub use chess::BoardStatus;

//Having this link hardcoded is bad, I should it fix later
pub const NEW_CHESS_GAME_LINK: &str = "https://www.chess.com/dynboard?fen=rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR&board=bases&piece=classic&size=3&coordinates=1";

pub struct ChessGames;

impl TypeMapKey for ChessGames {
    type Value = Mutex<Vec<ChessGame>>;
}

pub struct ChessGame {
    pub white_id: u64,
    pub black_id: u64,
    board: Board,
    pub show_coordinates: bool,
    pub board_flips: bool,
}

impl ChessGame {
    pub fn new_game_random_sides(player1_id: u64, player2_id: u64) -> Self {
        let mut rng = thread_rng();
        if rng.gen_bool(0.5) {
            Self {
                white_id: player1_id,
                black_id: player2_id,
                board: Board::default(),
                show_coordinates: true,
                board_flips: false,
            }
        }
        else {
            Self {
                white_id: player2_id,
                black_id: player1_id,
                board: Board::default(),
                show_coordinates: true,
                board_flips: false,
            }
        }
    }

    pub const fn has_user(&self, id: u64) -> bool {
        self.white_id == id || self.black_id == id
    }

    pub fn id_to_move(&self) -> u64 {
        match self.board.side_to_move() {
            Color::White => self.white_id,
            Color::Black => self.black_id
        }
    }

    pub fn make_move(&mut self, selected_move: ChessMove) {
        self.board = self.board.make_move_new(selected_move);
    }

    pub fn make_move_from_str(&mut self, move_str: &str) -> Result<(), chess::Error> {
        let selected_move_result = match ChessMove::from_str(move_str) {
            Ok(selected_move) => {
                if self.board.legal(selected_move) {
                    Ok(selected_move)
                }
                else {
                    //The move is invalid if it is illegal
                    Err(chess::Error::InvalidUciMove)
                }
            }
            Err(_) => {
                //If the move is invalid UCI attempt to get SAN
                ChessMove::from_san(&self.board, move_str)
            }
        };
        
        return match selected_move_result {
            Ok(selected_move) => {
                self.make_move(selected_move);
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    pub fn generate_hodgey_move(&mut self) -> Option<ChessMove> {
        let moves = MoveGen::new_legal(&self.board);
        moves.choose(&mut thread_rng())
    }

    pub fn is_in_check(&self) -> bool {
        self.board.checkers().popcnt() > 0
    }

    pub fn to_link(&self) -> String {
        let board_str = self.board.to_string();
        let fen = board_str.split(' ').next().unwrap();
        let mut result = format!("https://www.chess.com/dynboard?fen={fen}&board=bases&piece=classic&size=3");

        if self.show_coordinates {
            result += "&coordinates=1";
        }

        if self.board_flips && self.board.side_to_move() == Color::Black {
            result += "&flip=1";
        }

        result
    }

    pub fn status(&self) -> BoardStatus {
        self.board.status()
    }
}