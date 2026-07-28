use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use fenix::{Action, Coord, GamePhase, Player};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::Mutex;

struct AppState {
    pub game: fenix::Game,
}

#[derive(Deserialize)]
struct MoveIntent {
    from: (u8, u8),
    to: (u8, u8),
}

#[tokio::main]
async fn main() {
    let app_state = Arc::new(Mutex::new(AppState {
        game: fenix::Game::default(),
    }));

    let app = Router::new()
        .route("/state", get(state))
        .route("/move", post(play_move))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn phase_name(phase: &GamePhase) -> &str {
    match phase {
        GamePhase::Setup => "Setup",
        GamePhase::Normal => "Normal",
        GamePhase::ReconstructGeneral => "ReconstructGeneral",
        GamePhase::ReconstructKing => "ReconstructKing",
        GamePhase::ForcedCapture { .. } => "ForcedCapture",
        GamePhase::GameOver(_) => "GameOver",
    }
}

fn action_to_json(action: &Action) -> Value {
    let (fx, fy) = action.from().xy();
    let (tx, ty) = action.to().xy();
    json!({
        "from": [fx, fy],
        "to": [tx, ty],
    })
}

async fn state(State(state): State<Arc<Mutex<AppState>>>) -> Json<Value> {
    let game = &state.lock().await.game;

    let fen = game.board().to_fen();
    let turn = match game.side_to_play() {
        Player::Red => "Red",
        Player::Black => "Black",
    };
    let phase = phase_name(game.phase());
    let legal_moves: Vec<Value> = game.legal_actions().iter().map(action_to_json).collect();

    Json(json!({
        "fen": fen,
        "turn": turn,
        "phase": phase,
        "legal_moves": legal_moves,
    }))
}

async fn play_move(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(move_intent): Json<MoveIntent>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut guard = state.lock().await;
    let game = &mut guard.game;

    let from = Coord::from_xy(move_intent.from.0, move_intent.from.1)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))))?;
    let to = Coord::from_xy(move_intent.to.0, move_intent.to.1)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))))?;

    game.play_move(from, to)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))))?;

    let fen = game.board().to_fen();
    let turn = match game.side_to_play() {
        Player::Red => "Red",
        Player::Black => "Black",
    };
    let phase = phase_name(game.phase());
    let legal_moves: Vec<Value> = game.legal_actions().iter().map(action_to_json).collect();

    Ok(Json(json!({
        "fen": fen,
        "turn": turn,
        "phase": phase,
        "legal_moves": legal_moves,
    })))
}
