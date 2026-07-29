use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::{Result, bail};

use crate::{
    action::{Action, actions_for},
    board::{Board, Coord},
    capture::{capture_options, capture_options_from},
    square::Square,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy)]
pub enum GamePhase {
    Setup,
    Normal,
    ReconstructGeneral,
    ReconstructKing,
    ForcedCapture { from: Coord },
    GameOver(GameOutcome),
}

#[derive(Debug, Clone, Copy)]
pub enum GameOutcome {
    Win { winner: Player, reason: WinReason },
    Draw { reason: DrawReason },
}

#[derive(Debug, Clone, Copy)]
pub enum DrawReason {
    ThreefoldRepetition,
}

#[derive(Debug, Clone, Copy)]
pub enum WinReason {
    KingLost,
    ThreefoldRepetion,
}

#[derive(Debug, Clone, Copy)]
pub struct GameState {
    board: Board,
    turn_count: u32,
    side_to_play: Player,
    phase: GamePhase,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            board: Board::starting(),
            turn_count: 0,
            side_to_play: Player::Red,
            phase: GamePhase::Setup,
        }
    }
}

impl GameState {
    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn turn_count(&self) -> u32 {
        self.turn_count
    }

    pub fn side_to_play(&self) -> Player {
        self.side_to_play
    }

    pub fn phase(&self) -> &GamePhase {
        &self.phase
    }

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

    pub fn apply_action(mut self, action: Action) -> Self {
        match action {
            Action::Move { .. } | Action::Upgrade { .. } => {
                self.board = self.board.apply_action(&action);
                self.advance_turn(None)
            }
            Action::Capture { from, to, captured } => {
                let piece = self.board.at(captured);
                self.board = self.board.remove_piece(captured).move_piece(from, to);

                if !capture_options_from(&self.board, to).is_empty() {
                    self.phase = GamePhase::ForcedCapture { from: to };
                    return self;
                }

                self.advance_turn(Some(piece.kind()))
            }
        }
    }

    fn advance_turn(mut self, captured_kind: Option<u8>) -> Self {
        self.side_to_play = self.side_to_play.next();
        self.turn_count += 1;

        if self.turn_count < 10 {
            self.phase = GamePhase::Setup;
            return self;
        }

        self.phase = match captured_kind {
            Some(Square::KING) => GamePhase::ReconstructKing,
            Some(Square::GENERAL) => GamePhase::ReconstructGeneral,
            _ => GamePhase::Normal,
        };

        self
    }

    fn setup_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        actions_for(&self.board, self.side_to_play, &mut actions);

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
        actions_for(&self.board, self.side_to_play, &mut actions);

        actions
            .into_iter()
            .filter(|a| !matches!(a, Action::Upgrade { .. }))
            .collect()
    }

    fn reconstruct_general(&self) -> Vec<Action> {
        let captures = capture_options(&self.board, self.side_to_play);

        if !captures.is_empty() {
            return captures;
        }

        let mut actions = Vec::new();

        actions_for(&self.board, self.side_to_play, &mut actions);
        actions
            .into_iter()
            .filter(|a| {
                matches!(
                    a,
                    Action::Move { .. }
                        | Action::Upgrade {
                            upgrade: Square::GENERAL,
                            ..
                        }
                )
            })
            .collect()
    }

    fn reconstruct_king(&self) -> Vec<Action> {
        let mut actions = Vec::new();

        actions_for(&self.board, self.side_to_play, &mut actions);

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

pub struct Game {
    state: GameState,
    history: Vec<u64>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            state: GameState::default(),
            history: Vec::new(),
        }
    }
}

impl Game {
    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn board(&self) -> &Board {
        self.state.board()
    }

    pub fn turn_count(&self) -> u32 {
        self.state.turn_count()
    }

    pub fn side_to_play(&self) -> Player {
        self.state.side_to_play()
    }

    pub fn phase(&self) -> &GamePhase {
        self.state.phase()
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        self.state.legal_actions()
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
        self.state = self.state.apply_action(action);

        if matches!(self.state.phase, GamePhase::ForcedCapture { .. }) {
            return;
        }

        if let GamePhase::ReconstructKing = self.state.phase {
            if self.state.legal_actions().is_empty() {
                self.state.phase = GamePhase::GameOver(GameOutcome::Win {
                    winner: self.state.side_to_play().next(),
                    reason: WinReason::KingLost,
                });
            }
        }

        let hash = self.state_hash();
        self.history.push(hash);

        if self.history.iter().filter(|&&h| h == hash).count() >= 3 {
            self.state.phase = GamePhase::GameOver(GameOutcome::Draw {
                reason: DrawReason::ThreefoldRepetition,
            });
        }
    }

    fn state_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.state.board.hash(&mut s);
        self.state.side_to_play.hash(&mut s);
        s.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Red captures Black's King, Black must reconstruct.
    #[test]
    fn capture_king_triggers_reconstruct() {
        let mut board = Board::default();

        // Red General at (4,4) = 40
        board.squares[40] = Square::new(Square::RED, Square::GENERAL).unwrap();
        // Black King at (3,4) = 39, landing at (2,4) = 38
        board.squares[39] = Square::new(Square::BLACK, Square::KING).unwrap();
        // Black General at (3,1) = 12 for reconstruct
        board.squares[12] = Square::new(Square::BLACK, Square::GENERAL).unwrap();
        // Black Soldier at (3,2) = 21 — adjacent below General
        board.squares[21] = Square::new(Square::BLACK, Square::SOLDIER).unwrap();

        let mut game = Game {
            state: GameState {
                board,
                turn_count: 10,
                side_to_play: Player::Red,
                phase: GamePhase::Normal,
            },
            history: vec![],
        };

        // Red General at 40 captures Black King at 39, lands at 38
        game.play_move(Coord::new(40).unwrap(), Coord::new(38).unwrap())
            .unwrap();

        // After capture: side flips to Black
        assert_eq!(game.side_to_play(), Player::Black);
        // Phase should be ReconstructKing
        assert!(matches!(game.phase(), GamePhase::ReconstructKing));

        // Black should have at least one King-Upgrade legal
        let actions = game.legal_actions();
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::Upgrade {
                upgrade: Square::KING,
                ..
            }
        )));

        // Black reconstructs: Soldier at 21 upgrades General at 12 to King
        game.play_move(Coord::new(21).unwrap(), Coord::new(12).unwrap())
            .unwrap();

        // (3,1) should now be a Black King
        let king_sq = game.board().at(Coord::new(12).unwrap());
        assert_eq!(king_sq.kind(), Square::KING);
        assert_eq!(king_sq.color(), Square::BLACK);

        // Turn should be Red again, phase Normal
        assert_eq!(game.side_to_play(), Player::Red);
        assert!(matches!(game.phase(), GamePhase::Normal));
    }

    /// Red captures Black's King but Black has no Soldier+General pair → GameOver.
    #[test]
    fn capture_king_no_reconstruct_loses() {
        let mut board = Board::default();

        // Red General at (4,4) = 40
        board.squares[40] = Square::new(Square::RED, Square::GENERAL).unwrap();
        // Black King at (3,4) = 39
        board.squares[39] = Square::new(Square::BLACK, Square::KING).unwrap();
        // Black pieces placed far away so Red General can't chain-capture them
        board.squares[70] = Square::new(Square::BLACK, Square::SOLDIER).unwrap(); // (7,7)
        board.squares[71] = Square::new(Square::BLACK, Square::SOLDIER).unwrap(); // (8,7)

        let mut game = Game {
            state: GameState {
                board,
                turn_count: 10,
                side_to_play: Player::Red,
                phase: GamePhase::Normal,
            },
            history: vec![],
        };

        // Capture King
        game.play_move(Coord::new(40).unwrap(), Coord::new(38).unwrap())
            .unwrap();

        // Turn should be Black, phase GameOver — Black can't reconstruct
        assert_eq!(game.side_to_play(), Player::Black);
        assert!(matches!(
            game.phase(),
            GamePhase::GameOver(GameOutcome::Win {
                winner: Player::Red,
                reason: WinReason::KingLost,
            })
        ));
    }
}
