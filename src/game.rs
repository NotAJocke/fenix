use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::{Result, bail};

use crate::{
    action::{Action, action_for},
    board::{Board, Coord},
    capture::{capture_options, capture_options_from},
    square::Square,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash)]
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
}

#[derive(Debug)]
pub enum GamePhase {
    Setup,
    Normal,
    ReconstructGeneral,
    ReconstructKing,
    ForcedCapture { from: Coord },
    GameOver(GameOutcome),
}

#[derive(Debug)]
pub enum GameOutcome {
    Win { winner: Player, reason: WinReason },
    Draw { reason: DrawReason },
}

#[derive(Debug)]
pub enum DrawReason {
    ThreefoldRepetition,
}

#[derive(Debug)]
pub enum WinReason {
    KingLost,
    ThreefoldRepetion,
}

pub struct Game {
    pub board: Board,
    pub turn_count: u32,
    pub side_to_play: Player,
    pub phase: GamePhase,
    pub history: Vec<u64>,
    pub king_was_captured: bool,
    pub general_was_captured: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            board: Board::starting(),
            turn_count: 0,
            side_to_play: Player::Red,
            phase: GamePhase::Setup,
            history: Vec::new(),
            king_was_captured: false,
            general_was_captured: false,
        }
    }
}

impl Game {
    pub fn legal_actions(&self) -> Vec<Action> {
        match self.phase {
            GamePhase::Setup => self.setup_actions(),
            GamePhase::Normal => self.normal_actions(),
            GamePhase::ReconstructGeneral => self.reconstruct_general(),
            GamePhase::ReconstructKing => self.reconstruct_king(),
            GamePhase::ForcedCapture { .. } => self.forced_capture(),
            GamePhase::GameOver(_) => vec![],
        }
    }

    pub fn play_move(&mut self, from: Coord, to: Coord) -> Result<Action> {
        let legals = self.legal_actions();

        let action = legals
            .into_iter()
            .find(|a| a.from() == from && a.to() == to);

        let Some(action) = action else {
            bail!("Not a legal move")
        };

        self.apply_action(action);

        Ok(action)
    }

    fn apply_action(&mut self, action: Action) {
        match action {
            Action::Move { from, to } => self.board = self.board.move_piece(from, to),
            Action::Upgrade { from, to, .. } => {
                self.board = self.board.remove_piece(from).upgrade_piece(to);
            }
            Action::Capture { from, to, captured } => {
                let piece = self.board.at(captured);
                self.board = self.board.remove_piece(captured).move_piece(from, to);

                match piece.kind() {
                    Square::KING => self.king_was_captured = true,
                    Square::GENERAL => self.general_was_captured = true,
                    _ => {}
                }

                let more_captures = self.check_chain_captures(to);
                if more_captures {
                    return;
                }
            }
        }

        self.advance_turn();
    }

    fn check_chain_captures(&mut self, from: Coord) -> bool {
        let more_captures = capture_options_from(&self.board, from);

        if more_captures.is_empty() {
            return false;
        }

        self.phase = GamePhase::ForcedCapture { from };

        true
    }

    fn advance_turn(&mut self) {
        self.side_to_play = self.side_to_play.next();
        self.turn_count += 1;

        if self.turn_count < 10 {
            self.phase = GamePhase::Setup;
            return;
        }

        if self.king_was_captured {
            self.phase = GamePhase::ReconstructKing;

            let legals = self.legal_actions();

            if legals.is_empty() {
                self.phase = GamePhase::GameOver(GameOutcome::Win {
                    winner: self.side_to_play.next(),
                    reason: WinReason::KingLost,
                })
            }
        } else if self.general_was_captured {
            self.phase = GamePhase::ReconstructGeneral;
        } else {
            self.phase = GamePhase::Normal;
        }

        self.general_was_captured = false;
        self.king_was_captured = false;

        let hash = self.state_hash();
        self.history.push(hash);

        if self.history.iter().filter(|&&h| h == hash).count() >= 3 {
            self.phase = GamePhase::GameOver(GameOutcome::Draw {
                reason: DrawReason::ThreefoldRepetition,
            });
        }
    }

    fn state_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.board.hash(&mut s);
        self.side_to_play.hash(&mut s);
        s.finish()
    }

    fn setup_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        action_for(&self.board, self.side_to_play, &mut actions);

        actions
            .into_iter()
            .filter(|a| matches!(a, Action::Upgrade { .. }))
            .filter(|a| self.respects_material_caps(a))
            .collect()
    }

    fn normal_actions(&self) -> Vec<Action> {
        let captures = capture_options(&self.board, self.side_to_play);

        if !captures.is_empty() {
            return captures;
        }

        let mut actions = Vec::new();
        action_for(&self.board, self.side_to_play, &mut actions);

        actions
            .into_iter()
            // Upgrades are in their own game phases
            .filter(|a| !matches!(a, Action::Upgrade { .. }))
            .collect()
    }

    fn reconstruct_general(&self) -> Vec<Action> {
        let captures = capture_options(&self.board, self.side_to_play);

        if !captures.is_empty() {
            return captures;
        }

        let mut actions = Vec::new();

        action_for(&self.board, self.side_to_play, &mut actions);
        actions
            .into_iter()
            // Upgrades are in their own game phases
            .filter(|a| match a {
                Action::Move { .. } => true,
                Action::Upgrade {
                    upgrade: Square::GENERAL,
                    ..
                } => true,
                _ => false,
            })
            .collect()
    }

    fn reconstruct_king(&self) -> Vec<Action> {
        let mut actions = Vec::new();

        action_for(&self.board, self.side_to_play, &mut actions);

        actions
            .into_iter()
            .filter(|a| {
                matches!(
                    a,
                    Action::Upgrade {
                        upgrade: Square::KING,
                        ..
                    }
                )
            })
            .collect()
    }

    fn forced_capture(&self) -> Vec<Action> {
        let GamePhase::ForcedCapture { from } = self.phase else {
            panic!("Not in forced capture game phase while being in its handler")
        };

        capture_options_from(&self.board, from)
    }

    // Rule 4: max 3 Generals, max 1 King per player
    fn respects_material_caps(&self, action: &Action) -> bool {
        let Action::Upgrade { upgrade, .. } = action else {
            return true;
        };

        let mut generals = 0;
        let mut kings = 0;
        for sq in &self.board.squares {
            if sq.is_empty() || sq.color() != self.side_to_play as u8 {
                continue;
            }
            match sq.kind() {
                Square::GENERAL => generals += 1,
                Square::KING => kings += 1,
                _ => {}
            }
        }

        match *upgrade {
            Square::GENERAL => generals < 3,
            Square::KING => kings == 0,
            _ => true,
        }
    }
}
