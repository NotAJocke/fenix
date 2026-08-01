use crate::{Action, Board, Game, GameOutcome, GamePhase, GameState, Player};

pub trait Ai {
    fn choose_action(&self, game: &Game) -> Action;
}

pub struct Greedy;

impl Ai for Greedy {
    fn choose_action(&self, game: &Game) -> Action {
        game.legal_actions()
            .into_iter()
            .max_by_key(|a| evaluate(&game.board().apply_action(a), game.side_to_play()))
            .expect("game has legal moves")
    }
}

pub struct Minimax {
    pub depth: u32,
}

impl Ai for Minimax {
    fn choose_action(&self, game: &Game) -> Action {
        assert!(self.depth > 0, "Minimax depth must be at least 1");
        let state = *game.state();
        let mut best_action = None;
        let mut best_score = i32::MIN;
        for action in state.legal_actions() {
            let score = -negamax(
                state.apply_action(action),
                self.depth - 1,
                i32::MIN + 1,
                i32::MAX,
            );
            if score > best_score {
                best_score = score;
                best_action = Some(action);
            }
        }
        best_action.expect("game has legal moves")
    }
}

fn negamax(state: GameState, depth: u32, mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return evaluate(state.board(), state.side_to_play());
    }

    let actions = state.legal_actions();

    if actions.is_empty() {
        return match state.phase() {
            GamePhase::GameOver(outcome) => match outcome {
                GameOutcome::Win { winner, .. } => {
                    if *winner == state.side_to_play() {
                        i32::MAX / 2
                    } else {
                        i32::MIN / 2
                    }
                }
                GameOutcome::Draw { .. } => 0,
            },
            _ => i32::MIN / 2,
        };
    }

    let mut best = i32::MIN;
    for action in actions {
        let score = -negamax(state.apply_action(action), depth - 1, -beta, -alpha);
        if score > best {
            best = score;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }
    best
}

pub fn evaluate(board: &Board, player: Player) -> i32 {
    let mut score = 0i32;
    for (i, &sq) in board.squares.iter().enumerate() {
        if sq.is_empty() {
            continue;
        }
        let center = center_bonus(i);
        if sq.color() == player as u8 {
            score += sq.weight() as i32 + center;
        } else {
            score -= sq.weight() as i32 + center;
        }
    }
    score
}

fn center_bonus(index: usize) -> i32 {
    let x = index % 9;
    let y = index / 9;
    let dist = (x as i32 - 4).abs().max((y as i32 - 4).abs());
    match dist {
        0 => 2,
        1 => 1,
        _ => 0,
    }
}
