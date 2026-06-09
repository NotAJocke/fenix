use std::collections::HashSet;

use crate::{
    action::{Action, actions_from},
    board::{Board, Coord},
    game::Player,
};

pub fn capture_options(board: &Board, player: Player) -> Vec<Action> {
    let mut all_scores: Vec<(Action, u32)> = Vec::new();

    for i in 0..81 {
        let coord = Coord::new(i).unwrap();
        let square = board.at(coord);
        if square.is_empty() || square.color() != player as u8 {
            continue;
        }

        let scores = capture_scores(board, coord, &HashSet::new());
        all_scores.extend(scores);
    }

    best_actions(all_scores)
}

pub fn capture_options_from(board: &Board, from: Coord) -> Vec<Action> {
    let scores = capture_scores(board, from, &HashSet::new());
    best_actions(scores)
}

fn best_actions(scores: Vec<(Action, u32)>) -> Vec<Action> {
    if scores.is_empty() {
        return vec![];
    }

    let max_weight = scores.iter().map(|(_, w)| *w).max().unwrap();

    let mut seen = HashSet::new();
    scores
        .into_iter()
        .filter(|(a, w)| *w == max_weight && seen.insert(*a))
        .map(|(a, _)| a)
        .collect()
}

/// Returns (first_action, total_weight) for every complete capture path
/// starting from `from`.
///
/// The `visited` set tracks coords of pieces already captured — rule 6 says
/// "An enemy piece can only be jumped once in a single turn." We remove
/// captured pieces from the board immediately so they can't be jumped again,
/// and track them in `visited` as a redundant safety net.
fn capture_scores(board: &Board, from: Coord, visited: &HashSet<Coord>) -> Vec<(Action, u32)> {
    let mut actions = Vec::new();
    actions_from(board, from, &mut actions);

    let captures: Vec<Action> = actions
        .into_iter()
        .filter(|a| match a {
            Action::Capture { captured, .. } => !visited.contains(captured),
            _ => false,
        })
        .collect();

    if captures.is_empty() {
        return vec![];
    }

    let mut scores = Vec::new();

    for &capture in &captures {
        if let Action::Capture { from: cap_from, to: cap_to, captured } = capture {
            let new_board = board.clone().move_piece(cap_from, cap_to).remove_piece(captured);
            let weight = board.at(captured).weight() as u32;

            let mut new_visited = visited.clone();
            new_visited.insert(captured);

            let continuations = capture_scores(&new_board, cap_to, &new_visited);

            if continuations.is_empty() {
                scores.push((capture, weight));
            } else {
                for (_, cont_weight) in continuations {
                    scores.push((capture, weight + cont_weight));
                }
            }
        }
    }

    scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::game::Player;
    use crate::square::Square;

    fn make_piece(color: u8, kind: u8) -> Square {
        Square::new(color, kind).unwrap()
    }

    /// Two Soldiers at equal weight, both directions returned.
    #[test]
    fn two_equivalent_first_moves() {
        let mut board = Board::default();
        board.squares[40] = make_piece(Square::RED, Square::SOLDIER);
        board.squares[39] = make_piece(Square::BLACK, Square::SOLDIER);
        board.squares[41] = make_piece(Square::BLACK, Square::SOLDIER);

        let from = Coord::new(40).unwrap();
        let opts = capture_options_from(&board, from);

        assert_eq!(opts.len(), 2);
    }

    /// General captures two Soldiers in sequence, only first action returned.
    #[test]
    fn single_chain_returns_only_first_action() {
        let mut board = Board::default();
        board.squares[40] = make_piece(Square::RED, Square::GENERAL);
        board.squares[39] = make_piece(Square::BLACK, Square::SOLDIER);
        board.squares[37] = make_piece(Square::BLACK, Square::SOLDIER);

        let from = Coord::new(40).unwrap();
        let opts = capture_options_from(&board, from);

        assert_eq!(opts.len(), 1);
        assert_eq!(opts[0].to(), Coord::new(38).unwrap());
    }

    #[test]
    fn no_captures_returns_empty() {
        let board = Board::default();
        let opts = capture_options_from(&board, Coord::new(0).unwrap());
        assert!(opts.is_empty());
    }

    /// PLAN.md example: A->C->D weight=4, A->C->E weight=4, A->B->F weight=2
    /// capture_options returns [A->C] (only the optimal first action).
    #[test]
    fn plan_md_example() {
        let mut board = Board::default();
        board.squares[40] = make_piece(Square::RED, Square::GENERAL);    // A
        board.squares[39] = make_piece(Square::BLACK, Square::KING);     // C
        board.squares[37] = make_piece(Square::BLACK, Square::SOLDIER);  // D
        board.squares[29] = make_piece(Square::BLACK, Square::SOLDIER);  // E
        board.squares[31] = make_piece(Square::BLACK, Square::SOLDIER);  // B
        board.squares[13] = make_piece(Square::BLACK, Square::SOLDIER);  // F

        let opts = capture_options(&board, Player::Red);

        assert_eq!(opts.len(), 1);
        assert_eq!(opts[0].from(), Coord::new(40).unwrap());
        assert_eq!(opts[0].to(), Coord::new(38).unwrap());
    }

    /// Strict max-weight: Path A (weight 5) beats Path B (weight 3).
    /// Only Path A's first action should be returned.
    ///
    /// Setup:
    ///   Red General at (4,4)=40
    ///   Path A (up): Black King at (4,3)=31 (weight 3), then Black General at (4,1)=13 (weight 2)
    ///     → total weight = 5
    ///   Path B (left): Black King at (3,4)=39 (weight 3)
    ///     → total weight = 3
    #[test]
    fn strict_max_weight_picks_heavier_path() {
        let mut board = Board::default();
        board.squares[40] = make_piece(Square::RED, Square::GENERAL);     // A
        // Path A (up): King at (4,3), General at (4,1)
        board.squares[31] = make_piece(Square::BLACK, Square::KING);      // weight 3
        board.squares[13] = make_piece(Square::BLACK, Square::GENERAL);   // weight 2
        // Path B (left): King at (3,4)
        board.squares[39] = make_piece(Square::BLACK, Square::KING);      // weight 3

        let opts = capture_options(&board, Player::Red);

        assert_eq!(opts.len(), 1);
        // The heavier path goes up
        assert_eq!(opts[0].from(), Coord::new(40).unwrap());
        assert_eq!(opts[0].to(), Coord::new(22).unwrap()); // lands at (4,2)
    }

    /// Regression: a capture sequence that could revisit an already-jumped
    /// piece. The branch must be rejected.
    ///
    /// Setup: Red General at (4,4).
    ///   Black Soldier at (3,4) — capture to (2,4)
    ///   Black Soldier at (1,4) — capture to (0,4)
    ///   Black Soldier at (0,3) — from (0,4), could capture back toward (1,4)
    ///     but (1,4) was already captured → must NOT allow this branch.
    ///
    /// The only valid path is (4,4)→(2,4)→(0,4), total weight=2.
    /// From (0,4), capturing (0,3) would land at (0,2) — this is valid since
    /// (0,3) was NOT previously captured. Weight=3 total.
    ///
    /// But if we set up a piece that requires re-jumping (1,4), it must be blocked.
    /// Red General at (4,4), Black Soldier at (3,4), Black Soldier at (1,4),
    /// Black Soldier at (2,3).
    /// Path: (4,4)→(2,4) capturing (3,4), then from (2,4) can capture (1,4)
    ///   → land (0,4), weight=2
    ///   OR capture (2,3) → land (2,2), weight=2
    /// From (0,4), nothing more to capture. From (2,2), nothing more to capture.
    /// No re-jumping issue here. Let me make a tighter example.
    ///
    /// Better: Red General at (4,4), going left:
    ///   Black Soldier at (3,4), land (2,4)
    ///   Black Soldier at (1,4), land (0,4)
    /// Now from (0,4), going down: Black Soldier at (0,5), land (0,6)
    /// From (0,6), going right: (1,6) empty, (2,6) empty... no re-jump possible.
    ///
    /// The re-jump scenario requires the piece to be reachable again:
    /// Red General at (2,2), Black Soldier at (1,2), land (0,2)
    /// From (0,2), Black Soldier at (0,1), land (0,0)
    /// From (0,0), going down: Black Soldier at (0,1) — but already captured!
    /// This branch must be rejected.
    #[test]
    fn rejects_revisiting_already_captured_piece() {
        let mut board = Board::default();
        // Red General at (2,2) = 2*9+2 = 20
        board.squares[20] = make_piece(Square::RED, Square::GENERAL);
        // Black Soldier at (1,2) = 2*9+1 = 19
        board.squares[19] = make_piece(Square::BLACK, Square::SOLDIER);
        // Black Soldier at (0,1) = 1*9+0 = 9
        board.squares[9] = make_piece(Square::BLACK, Square::SOLDIER);

        // Path: (2,2)→(0,2) capturing (1,2), then (0,2)→(0,0) capturing (0,1)
        // From (0,0), going down: (0,1) is gone (captured) → no further captures
        // Total weight = 1+1 = 2
        let from = Coord::new(20).unwrap();
        let scores = capture_scores(&board, from, &HashSet::new());

        // Verify we get exactly one path with weight 2
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].1, 2);

        // The first action captures (1,2) and lands at (0,2)
        assert_eq!(scores[0].0.to(), Coord::new(18).unwrap()); // (0,2) = 2*9+0 = 18
    }
}
