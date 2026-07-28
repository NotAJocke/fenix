use std::time::Duration;
use std::{convert::Infallible, sync::Arc};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use fenix::{
    Action, Coord, DrawReason, Game, GameOutcome, GamePhase, Player, WinReason,
};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::{Mutex, broadcast};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

struct AppState {
    pub game: fenix::Game,
    pub tx: broadcast::Sender<String>,
}

#[derive(Deserialize)]
struct MoveIntent {
    from: (u8, u8),
    to: (u8, u8),
}

#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel(16);
    let app_state = Arc::new(Mutex::new(AppState {
        game: fenix::Game::default(),
        tx,
    }));

    let app = Router::new()
        .route("/state", get(state))
        .route("/move", post(play_move))
        .route("/events", get(events))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn state_json(game: &Game) -> Value {
    let fen = game.board().to_fen();
    let turn = match game.side_to_play() {
        Player::Red => "Red",
        Player::Black => "Black",
    };
    let phase = phase_name(game.phase());
    let legal_moves: Vec<Value> = game.legal_actions().iter().map(action_to_json).collect();

    let outcome = match game.phase() {
        GamePhase::GameOver(outcome) => {
            let o = match outcome {
                GameOutcome::Win { winner, reason } => {
                    let w = match winner {
                        Player::Red => "Red",
                        Player::Black => "Black",
                    };
                    let r = match reason {
                        WinReason::KingLost => "king_lost",
                        WinReason::ThreefoldRepetion => "threefold_repetition",
                    };
                    json!({"winner": w, "reason": r})
                }
                GameOutcome::Draw { reason } => {
                    let r = match reason {
                        DrawReason::ThreefoldRepetition => "threefold_repetition",
                    };
                    json!({"draw": true, "reason": r})
                }
            };
            Some(o)
        }
        _ => None,
    };

    json!({
        "fen": fen,
        "turn": turn,
        "phase": phase,
        "outcome": outcome,
        "legal_moves": legal_moves,
    })
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
    Json(state_json(&state.lock().await.game))
}

async fn play_move(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(move_intent): Json<MoveIntent>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut guard = state.lock().await;
    let game = &mut guard.game;

    let from = Coord::from_xy(move_intent.from.0, move_intent.from.1).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )
    })?;
    let to = Coord::from_xy(move_intent.to.0, move_intent.to.1).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    game.play_move(from, to).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    let json = state_json(game);
    let _ = guard.tx.send(json.to_string());

    Ok(Json(json))
}

async fn events(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.lock().await.tx.subscribe();

    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(data) => Some(Ok(Event::default().data(data))),
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
