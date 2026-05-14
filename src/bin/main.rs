use fenix2::board::{self, Board};

fn main() {
    let board = Board::from_fen(board::STARTING_FEN).unwrap();

    println!("{board}");

    println!("{}", board.to_fen());
}
