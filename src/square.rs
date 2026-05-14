use crate::Result;

/// A square is either a piece from the red or black or nothing at all.
///
/// It's represented on a u8 using the 3 last bytes only.
///
/// 2 last bytes are piece type:
/// 00 -> Nothin
/// 01 -> Soldier
/// 10 -> General
/// 11 -> King
///
/// The 3rd last byte is piece color:
/// 0 -> red
/// 1 -> black
///
/// Ex: 0b00000101 = 5 -> Black soldier
#[derive(Debug, Clone, Copy)]
pub struct Square(pub(crate) u8);

impl Square {
    pub(crate) const EMPTY: Square = Square(0);

    const TYPE_MASK: u8 = 0b011;
    const COLOR_MASK: u8 = 0b100;

    const SOLDIER: u8 = 0b01;
    const GENERAL: u8 = 0b10;
    const KING: u8 = 0b11;

    pub fn new(byte: u8) -> Result<Self> {
        if byte > 7 || byte == 4 {
            return Err("Piece not exist".into());
        }

        Ok(Self(byte))
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn piece_type(self) -> u8 {
        self.0 & Self::TYPE_MASK
    }

    pub fn color(self) -> u8 {
        self.0 & Self::COLOR_MASK
    }

    pub fn upgraded(self) -> Option<Self> {
        if self.is_empty() {
            return None;
        }

        let next_type = match self.piece_type() {
            Self::SOLDIER => Self::GENERAL,
            Self::GENERAL => Self::KING,
            _ => return None, // Already King
        };

        Some(Square(next_type | self.color()))
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self.0 {
            0 => "..",
            1 => "RS",
            2 => "RG",
            3 => "RK",
            5 => "BS",
            6 => "BG",
            7 => "BK",
            _ => unreachable!(),
        };

        write!(f, "{}", repr)
    }
}
