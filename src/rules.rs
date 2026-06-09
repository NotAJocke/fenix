use crate::{
    action::Action,
    game::{Game, Player},
    square::Square,
};

/// This should be used once pseudo legals have been generated
/// and must be filtered.
pub fn is_legal(game: &Game, action: Action) -> bool {
    let rules = [
        player_owns_piece,
        setup_phase_allows,
        general_limit_allows,
        king_limit_allows,
        forces_king_when_missing,
    ];

    rules.iter().all(|f| f(game, action))
}

// HELPERS \\
fn is_setup_phase(game: &Game) -> bool {
    game.turn_count < 10
}

// RULES \\
fn setup_phase_allows(game: &Game, action: Action) -> bool {
    if !is_setup_phase(game) {
        return true;
    }

    match action {
        Action::Upgrade { .. } => true,
        _ => false,
    }
}

fn player_owns_piece(game: &Game, action: Action) -> bool {
    match game.side_to_play {
        Player::Red => game.board.at(action.from()).color() == Square::RED,
        Player::Black => game.board.at(action.from()).color() == Square::BLACK,
    }
}

fn general_limit_allows(game: &Game, action: Action) -> bool {
    match action {
        Action::Upgrade {
            upgrade: Square::GENERAL,
            ..
        } => {
            let player_index = game.side_to_play.index();
            let general_index = Square::GENERAL_INDEX;

            game.materials[player_index].counts[general_index] < 3
        }
        _ => true,
    }
}

fn king_limit_allows(game: &Game, action: Action) -> bool {
    match action {
        Action::Upgrade {
            upgrade: Square::KING,
            ..
        } => {
            let player_index = game.side_to_play.index();
            let king_index = Square::KING_INDEX;

            game.materials[player_index].counts[king_index] == 0
        }
        _ => true,
    }
}

fn forces_king_when_missing(game: &Game, action: Action) -> bool {
    if is_setup_phase(game) {
        return true;
    }

    let player_index = game.side_to_play.index();
    let king_index = Square::KING_INDEX;

    let has_king = game.materials[player_index].counts[king_index] > 0;
    if has_king {
        return true;
    }

    matches!(
        action,
        Action::Upgrade {
            upgrade: Square::KING,
            ..
        }
    )
}

// #[cfg(test)]
// mod rules_tests {
//     use crate::{action::Action, board::Coord, game::Game, square::Square};

//     #[test]
//     fn player_owns_piece() {
//         let game = Game::default();

//         assert!(super::player_owns_piece(
//             &game,
//             // Red piece
//             Action::Upgrade {
//                 from: Coord::from_xy(7, 3).unwrap(),
//                 to: Coord::from_xy(8, 3).unwrap(),
//                 upgrade: Square::GENERAL,
//             }
//         ));

//         assert!(!super::player_owns_piece(
//             &game,
//             // Black piece
//             Action::Upgrade {
//                 from: Coord::from_xy(0, 0).unwrap(),
//                 to: Coord::from_xy(0, 1).unwrap(),
//                 upgrade: Square::GENERAL,
//             }
//         ));
//     }

//     #[test]
//     fn setup_phase() {
//         let game = Game::default();

//         let legals = [Action::Upgrade {
//             from: Coord::from_xy(7, 3).unwrap(),
//             to: Coord::from_xy(8, 3).unwrap(),
//             upgrade: Square::GENERAL,
//         }];

//         for legal in legals {
//             assert!(super::setup_phase_allows(&game, legal));
//         }

//         let illegals = [
//             Action::Move {
//                 from: Coord::from_xy(7, 3).unwrap(),
//                 to: Coord::from_xy(8, 3).unwrap(),
//             },
//             Action::Capture {
//                 from: Coord::from_xy(7, 3).unwrap(),
//                 to: Coord::from_xy(8, 3).unwrap(),
//                 captured: Coord::new(0).unwrap(),
//             },
//         ];

//         for illegal in illegals {
//             assert!(super::setup_phase_allows(&game, illegal))
//         }
//     }
// }
