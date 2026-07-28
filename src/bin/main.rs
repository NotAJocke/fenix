use std::io::{self, Write};

use anyhow::Result;
use fenix::{Coord, Game};

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

    loop {
        println!("{}", game.board());
        println!(
            "Turn {} — {:?} to play ({:?})",
            game.turn_count(), game.side_to_play(), game.phase()
        );

        let legals = game.legal_actions();
        if legals.is_empty() {
            break;
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
        game.turn_count(), game.phase()
    );
}
