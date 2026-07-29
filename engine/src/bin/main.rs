use std::io::{self, Write};

use anyhow::Result;
use fenix::ai::{Ai, Greedy, Minimax};
use fenix::{Coord, Game, Player};

fn read_coord(label: &str) -> Result<Coord> {
    print!("{}", label);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let parts: Vec<u8> = buf.split_whitespace().flat_map(|s| s.parse()).collect();
    let [x, y]: [u8; 2] = parts
        .try_into()
        .map_err(|_| anyhow::anyhow!("Enter two numbers: x y"))?;
    Coord::from_xy(x, y)
}

fn main() {
    let mut game = Game::default();

    let mut red_ai = false;
    let mut black_ai = false;
    let mut depth = 1u32;

    let mut args = std::env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ai" => {
                let who = args.peek().map(|s| s.as_str());
                match who {
                    Some("red") => {
                        red_ai = true;
                        args.next();
                    }
                    Some("black") => {
                        black_ai = true;
                        args.next();
                    }
                    Some("both") => {
                        red_ai = true;
                        black_ai = true;
                        args.next();
                    }
                    _ => {
                        red_ai = true;
                        black_ai = true;
                    }
                }
            }
            "--depth" => {
                depth = args
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
            }
            _ => {}
        }
    }

    let ai: Box<dyn Ai> = if depth > 1 {
        Box::new(Minimax { depth })
    } else {
        Box::new(Greedy)
    };

    loop {
        println!("{}", game.board());
        println!(
            "Turn {} — {:?} to play ({:?})",
            game.turn_count(),
            game.side_to_play(),
            game.phase()
        );

        let legals = game.legal_actions();
        if legals.is_empty() {
            break;
        }

        let is_ai = match game.side_to_play() {
            Player::Red => red_ai,
            Player::Black => black_ai,
        };

        if is_ai {
            let action = ai.choose_action(&game);
            println!("AI plays: {}", action);
            game.play_move(action.from(), action.to()).unwrap();
            continue;
        }

        loop {
            let from = match read_coord("From (x y): ") {
                Ok(c) => c,
                Err(e) => {
                    println!("{e}");
                    continue;
                }
            };
            let to = match read_coord("To   (x y): ") {
                Ok(c) => c,
                Err(e) => {
                    println!("{e}");
                    continue;
                }
            };

            match game.play_move(from, to) {
                Ok(_) => break,
                Err(e) => println!("{e}"),
            }
        }
    }

    println!("{}", game.board());
    println!(
        "Game over after {} turns: {:?}",
        game.turn_count(),
        game.phase()
    );
}
