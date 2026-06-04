use crate::{
    action::{Action, MoveIntent},
    board::{self, Board, Coord},
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Player {
    Red = 0,
    Black = 0b100,
}

impl Player {
    pub fn next(self) -> Self {
        match self {
            Player::Red => Player::Black,
            Player::Black => Player::Red,
        }
    }

    pub fn index(self) -> usize {
        if self as u8 > 0 { 1 } else { 0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub(crate) counts: [u8; 3],
}

#[derive(Debug, Clone)]
pub struct CaptureSeq {
    acting_piece: Coord,
    next_captures: Vec<Action>,
}

pub struct Game {
    pub(crate) board: Board,
    pub(crate) turn_count: u32,
    pub(crate) side_to_play: Player,
    pub(crate) materials: [Material; 2],
    pub(crate) history: Vec<Action>,
    pub(crate) active_capture: Option<CaptureSeq>,
    pub(crate) lost_general_last_round: bool,
    pub(crate) available_actions: Vec<Action>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            board: Board::from_fen(board::STARTING_FEN).unwrap(),
            turn_count: 0,
            side_to_play: Player::Red,
            materials: [Material { counts: [28, 0, 0] }; 2],
            history: Vec::new(),
            active_capture: None,
            lost_general_last_round: false,
            available_actions: Vec::new(),
        }
    }
}
