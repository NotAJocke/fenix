use crate::Result;
use crate::square::Square;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord(u8);

impl Coord {
    pub fn new(coord: u8) -> Result<Self> {
        if coord >= 81 {
            return Err("Out of bounds".into());
        }

        Ok(Self(coord))
    }

    pub fn from_xy(x: u8, y: u8) -> Result<Self> {
        if x >= 9 || y >= 9 {
            return Err("Out of bounds".into());
        }

        Ok(Self(y * 9 + x))
    }

    pub fn xy(&self) -> (u8, u8) {
        (self.0 % 9, self.0 / 9)
    }

    pub fn checked_offset(&self, dx: i8, dy: i8) -> Option<Coord> {
        let (x, y) = self.xy();
        let nx = (x as i8).checked_add(dx)?;
        let ny = (y as i8).checked_add(dy)?;

        if nx < 0 || nx >= 9 || ny < 0 || ny >= 9 {
            return None;
        }

        Self::from_xy(nx as u8, ny as u8).ok()
    }
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x, y) = self.xy();
        write!(f, "({x},{y})")
    }
}

pub const STARTING_FEN: &str =
    "sssssss2/ssssss3/sssss3S/ssss3SS/sss3SSS/ss3SSSS/s3SSSSS/3SSSSSS/2SSSSSSS";

#[derive(Debug, Clone)]
pub struct Board {
    pub squares: [Square; 81],
}

impl Default for Board {
    fn default() -> Self {
        Self {
            squares: [Square::EMPTY; 81],
        }
    }
}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Self> {
        let mut board = Self::default();
        let mut i = 0;

        for line in fen.split("/") {
            for c in line.chars() {
                if i >= 81 {
                    return Err("FEN string exceeds board capacity".into());
                }

                if let Ok(sq) = Square::from_char(c) {
                    board.squares[i] = sq;
                    i += 1;
                } else if let Some(n) = c.to_digit(10) {
                    i += n as usize;
                } else {
                    return Err(format!("Invalid character in FEN: {c}").into());
                }
            }
        }

        if i != 81 {
            return Err(format!("FEN provided {} squares, but 81 are required", i).into());
        }

        Ok(board)
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for row in 0..9 {
            let mut empty_count = 0;
            for col in 0..9 {
                let coord = Coord::from_xy(col, row).unwrap();
                let sq = self.at(coord);

                if sq.is_empty() {
                    empty_count += 1;
                } else {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    fen.push(sq.to_char());
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if row < 8 {
                fen.push('/');
            }
        }
        fen
    }

    pub fn at(&self, coord: Coord) -> Square {
        unsafe { *self.squares.get_unchecked(coord.0 as usize) }
    }

    pub fn is_occupied(&self, coord: Coord) -> bool {
        !self.at(coord).is_empty()
    }

    pub fn place_piece(mut self, coord: Coord, piece: Square) -> Self {
        self.squares[coord.0 as usize] = piece;
        self
    }

    pub fn remove_piece(mut self, coord: Coord) -> Self {
        self.squares[coord.0 as usize] = Square::EMPTY;
        self
    }

    pub fn move_piece(self, from: Coord, to: Coord) -> Self {
        let square = self.at(from);
        self.place_piece(to, square).remove_piece(from)
    }

    pub fn upgrade_piece(self, coord: Coord) -> Self {
        let square = self.at(coord);

        match square.upgraded() {
            Some(new_piece) => self.place_piece(coord, new_piece),
            None => self,
        }
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..9 {
            for col in 0..9 {
                let coord = Coord::from_xy(col, row).unwrap();
                let square = self.at(coord);
                write!(f, "{square}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
