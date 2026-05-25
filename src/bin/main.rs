use fenix2::{
    Result,
    action::actions_from,
    board::{self, Board, Coord},
    square::Square,
};

fn main() -> Result<()> {
    let board = Board::from_fen(board::STARTING_FEN).unwrap();
    // .place_piece(
    //     Coord::from_xy(4, 4)?,
    //     Square::new(Square::BLACK, Square::SOLDIER)?,
    // )
    // .place_piece(Coord::from_xy(3, 4)?, Square::from_char('g')?)
    // .place_piece(
    //     Coord::from_xy(5, 4)?,
    //     Square::new(Square::RED, Square::SOLDIER)?,
    // )
    // .place_piece(
    //     Coord::from_xy(7, 4)?,
    //     Square::new(Square::BLACK, Square::SOLDIER)?,
    // );

    println!("{board}");

    Ok(())
}
