use std::{fmt, ops::Add};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Tile {
    pub file: char,
    pub rank: char,
}

impl From<&str> for Tile {
    fn from(item: &str) -> Self {
        let item = item.trim();
        assert!(item.len() == 2);

        let mut iter = item.chars();
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();

        Tile { file, rank }
    }
}

// Move along the board using + (x,y)
impl Add<(i8, i8)> for Tile {
    type Output = Option<Tile>;

    fn add(self, direction: (i8, i8)) -> Self::Output {
        let x = direction.0;
        let y = direction.1;

        Tile::new(
            (self.file as i8 + x) as u8 as char,
            (self.rank as i8 + y) as u8 as char,
        )
    }
}

impl Tile {
    // for ray casting
    pub const UP: (i8, i8) = (0, 1);
    pub const DOWN: (i8, i8) = (0, -1);
    pub const RIGHT: (i8, i8) = (1, 0);
    pub const LEFT: (i8, i8) = (-1, 0);

    pub const UPRIGHT: (i8, i8) = (1, 1);
    pub const UPLEFT: (i8, i8) = (-1, 1);
    pub const DOWNRIGHT: (i8, i8) = (1, -1);
    pub const DOWNLEFT: (i8, i8) = (-1, -1);

    /* a tile can only exist within the range a1 - h8 */
    pub fn new(file: char, rank: char) -> Option<Tile> {
        if (file < 'a' || file > 'h') || rank < '1' || rank > '8' {
            None
        } else {
            Some(Tile { file, rank })
        }
    }

    pub fn to_index(&self) -> u8 {
        let x = (self.file as u8) - 97;
        let y = (self.rank as u8) - 49;

        (7 - y) * 8 + x
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.file.to_string() + &self.rank.to_string();
        write!(f, "{}", s)
    }
}
