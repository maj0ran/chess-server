use std::{fmt, ops::Add};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Tile {
    pub file: char,
    pub rank: char,
}

impl From<String> for Tile {
    fn from(item: String) -> Self {
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

        let f = Tile {
            file: (self.file as i8 + x) as u8 as char,
            rank: (self.rank as i8 + y) as u8 as char,
        };

        if (f.file < 'a' || f.file > 'h') || f.rank < '1' || f.rank > '8' {
            None
        } else {
            Some(f)
        }
    }
}

impl Tile {
    pub const UP: (i8, i8) = (0, 1);
    pub const DOWN: (i8, i8) = (0, -1);
    pub const RIGHT: (i8, i8) = (1, 0);
    pub const LEFT: (i8, i8) = (-1, 0);

    pub const UPRIGHT: (i8, i8) = (1, 1);
    pub const UPLEFT: (i8, i8) = (-1, 1);
    pub const DOWNRIGHT: (i8, i8) = (1, -1);
    pub const DOWNLEFT: (i8, i8) = (-1, -1);
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.file.to_string() + &self.rank.to_string();
        write!(f, "{}", s)
    }
}
