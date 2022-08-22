#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Field {
    pub file: char,
    pub rank: char,
}

impl From<String> for Field {
    fn from(item: String) -> Self {
        let item = item.trim();
        assert!(item.len() == 2);

        let mut iter = item.chars();
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();

        Field { file, rank }
    }
}

impl Field {
    pub fn up(&self) -> Self {
        Field {
            file: self.file,
            rank: (self.rank as u8 + 1) as char,
        }
    }

    pub fn down(&self) -> Self {
        Field {
            file: self.file,
            rank: (self.rank as u8 - 1) as char,
        }
    }

    pub fn left(&self) -> Self {
        Field {
            file: (self.file as u8 - 1) as char,
            rank: self.rank,
        }
    }

    pub fn right(&self) -> Self {
        Field {
            file: (self.file as u8 + 1) as char,
            rank: self.rank,
        }
    }
}
