use std::{
    fmt,
    ops::{Add, Sub},
};

use log::info;

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

impl From<u8> for Tile {
    fn from(value: u8) -> Self {
        let file = (value % 8 + 97) as char;
        let rank = (value / 8 + 49) as char;

        info!("{}{}", file, rank);

        Tile::new(file, rank).unwrap() // TODO: dirty
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

impl Sub<(i8, i8)> for Tile {
    type Output = Option<Tile>;

    fn sub(self, direction: (i8, i8)) -> Self::Output {
        let x = direction.0;
        let y = direction.1;

        Tile::new(
            (self.file as i8 - x) as u8 as char,
            (self.rank as i8 - y) as u8 as char,
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

    pub fn all() -> Vec<Tile> {
        vec![
            Tile::from("a8"),
            Tile::from("b8"),
            Tile::from("c8"),
            Tile::from("d8"),
            Tile::from("e8"),
            Tile::from("f8"),
            Tile::from("g8"),
            Tile::from("h8"),
            Tile::from("a7"),
            Tile::from("b7"),
            Tile::from("c7"),
            Tile::from("d7"),
            Tile::from("e7"),
            Tile::from("f7"),
            Tile::from("g7"),
            Tile::from("h7"),
            Tile::from("a6"),
            Tile::from("b6"),
            Tile::from("c6"),
            Tile::from("d6"),
            Tile::from("e6"),
            Tile::from("f6"),
            Tile::from("g6"),
            Tile::from("h6"),
            Tile::from("a5"),
            Tile::from("b5"),
            Tile::from("c5"),
            Tile::from("d5"),
            Tile::from("e5"),
            Tile::from("f5"),
            Tile::from("g5"),
            Tile::from("h5"),
            Tile::from("a4"),
            Tile::from("b4"),
            Tile::from("c4"),
            Tile::from("d4"),
            Tile::from("e4"),
            Tile::from("f4"),
            Tile::from("g4"),
            Tile::from("h4"),
            Tile::from("a3"),
            Tile::from("b3"),
            Tile::from("c3"),
            Tile::from("d3"),
            Tile::from("e3"),
            Tile::from("f3"),
            Tile::from("g3"),
            Tile::from("h3"),
            Tile::from("a2"),
            Tile::from("b2"),
            Tile::from("c2"),
            Tile::from("d2"),
            Tile::from("e2"),
            Tile::from("f2"),
            Tile::from("g2"),
            Tile::from("h2"),
            Tile::from("a1"),
            Tile::from("b1"),
            Tile::from("c1"),
            Tile::from("d1"),
            Tile::from("e1"),
            Tile::from("f1"),
            Tile::from("g1"),
            Tile::from("h1"),
        ]
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.file.to_string() + &self.rank.to_string();
        write!(f, "{}", s)
    }
}
