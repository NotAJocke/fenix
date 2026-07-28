use fenix2::game::Game;

fn main() {
    let mut game = Game::default();

    loop {
        let legals = game.legal_actions();

        if legals.is_empty() {
            break;
        }

        let idx = fastrand::usize(..legals.len());
        let action = legals[idx];
        game.play_move(action.from(), action.to()).unwrap();

        if cfg!(debug_assertions) {
            println!("{}", game.board)
        }
    }

    println!("Game ended after {} turns", game.turn_count);
    println!("Phase: {:?}", game.phase);
}
