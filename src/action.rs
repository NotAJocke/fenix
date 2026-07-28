use crate::{
    board::{Board, Coord},
    game::Player,
    square::Square,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Move {
        from: Coord,
        to: Coord,
    },
    Upgrade {
        from: Coord,
        to: Coord,
        upgrade: u8,
    },
    Capture {
        from: Coord,
        to: Coord,
        captured: Coord,
    },
}

impl Action {
    pub fn from(self) -> Coord {
        match self {
            Self::Move { from, .. } => from,
            Self::Upgrade { from, .. } => from,
            Self::Capture { from, .. } => from,
        }
    }

    pub fn to(self) -> Coord {
        match self {
            Self::Move { to, .. } => to,
            Self::Upgrade { to, .. } => to,
            Self::Capture { to, .. } => to,
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Move { from, to } => {
                write!(f, "Move from {} to {}", from, to)
            }
            Action::Upgrade { from, to, upgrade } => {
                write!(f, "Upgrade from {} to {} into {}", from, to, upgrade)
            }
            Action::Capture { from, to, captured } => {
                write!(f, "Capture from {} to {} taking {}", from, to, captured)
            }
        }
    }
}

pub fn action_for(board: &Board, player: Player, actions: &mut Vec<Action>) {
    board.squares.iter().enumerate().for_each(|(i, &square)| {
        if square.is_empty() || square.color() != player as u8 {
            return;
        }

        let coord =
            Coord::new(i as u8).expect("Iterate through board squares cannot be out of bounds.");
        actions_from(board, coord, actions);
    });
}

pub fn actions_from(board: &Board, from: Coord, actions: &mut Vec<Action>) {
    match board.at(from).kind() {
        Square::SOLDIER => soldier_candidates(board, from, actions),
        Square::GENERAL => general_candidates(board, from, actions),
        Square::KING => king_candidates(board, from, actions),
        _ => (),
    }
}

fn soldier_candidates(board: &Board, from: Coord, actions: &mut Vec<Action>) {
    let directions: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let self_piece = board.at(from);

    for (dx, dy) in directions {
        let target = from.checked_offset(dx, dy);

        let Some(target) = target else {
            continue;
        };

        let target_square = board.at(target);

        if target_square.is_empty() {
            actions.push(Action::Move { from, to: target });
            continue;
        }

        if target_square.color() == self_piece.color()
            && let Some(upgraded) = target_square.upgraded()
        {
            actions.push(Action::Upgrade {
                from,
                to: target,
                upgrade: upgraded.kind(),
            });
            continue;
        }

        if target_square.color() != self_piece.color()
            && let Some(next_target) = target.checked_offset(dx, dy)
            && board.at(next_target).is_empty()
        {
            actions.push(Action::Capture {
                from,
                to: next_target,
                captured: target,
            });
        }
    }
}

fn general_candidates(board: &Board, from: Coord, actions: &mut Vec<Action>) {
    let directions: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let self_piece = board.at(from);

    for (dx, dy) in directions {
        let mut maybe_target = from.checked_offset(dx, dy);
        let mut capturing = None;

        while let Some(target) = maybe_target {
            let target_square = board.at(target);

            if target_square.is_empty() {
                match capturing {
                    Some(captured) => actions.push(Action::Capture {
                        from,
                        to: target,
                        captured,
                    }),
                    None => actions.push(Action::Move { from, to: target }),
                }

                maybe_target = target.checked_offset(dx, dy);
                continue;
            }

            if target_square.color() != self_piece.color()
                && let Some(next_target) = target.checked_offset(dx, dy)
                && board.at(next_target).is_empty()
            {
                if capturing.is_some() {
                    break;
                }

                capturing = Some(target);
                actions.push(Action::Capture {
                    from,
                    to: next_target,
                    captured: target,
                });

                maybe_target = next_target.checked_offset(dx, dy);
                continue;
            }

            break;
        }
    }
}

fn king_candidates(board: &Board, from: Coord, actions: &mut Vec<Action>) {
    let directions: [(i8, i8); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let self_piece = board.at(from);

    for (dx, dy) in directions {
        let target = from.checked_offset(dx, dy);

        let Some(target) = target else {
            continue;
        };

        let target_square = board.at(target);

        if target_square.is_empty() {
            actions.push(Action::Move { from, to: target });
            continue;
        }

        if target_square.color() != self_piece.color()
            && let Some(next_target) = target.checked_offset(dx, dy)
            && board.at(next_target).is_empty()
        {
            actions.push(Action::Capture {
                from,
                to: next_target,
                captured: target,
            });
        }
    }
}
