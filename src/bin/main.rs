use fenix2::{
    Result,
    action::actions_from,
    board::{Board, Coord},
    square::Square,
};

fn main() -> Result<()> {
    let board = Board::default()
        // .place_piece(
        //     Coord::from_xy(4, 4)?,
        //     Square::new(Square::BLACK, Square::SOLDIER)?,
        // )
        .place_piece(Coord::from_xy(3, 4)?, Square::from_char('g')?)
        .place_piece(
            Coord::from_xy(5, 4)?,
            Square::new(Square::RED, Square::SOLDIER)?,
        )
        .place_piece(
            Coord::from_xy(7, 4)?,
            Square::new(Square::BLACK, Square::SOLDIER)?,
        );

    println!("{board}");

    let mut actions = Vec::new();
    actions_from(&board, Coord::from_xy(3, 4).unwrap(), &mut actions);

    for action in actions {
        println!("{action}");
    }

    Ok(())
}
