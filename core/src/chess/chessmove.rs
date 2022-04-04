use crate::tile::Tile;
use crate::*;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecialMove {
    Queen,
    Knight,
    Rook,
    Bishop,
    KingsideCastle,
    QueensideCastle,
}

impl fmt::Display for SpecialMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            SpecialMove::Queen => 'Q',
            SpecialMove::Knight => 'N',
            SpecialMove::Rook => 'R',
            SpecialMove::Bishop => 'B',
            _ => ' ',
        };
        write!(f, "{}", c)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChessMove {
    pub src: Tile,
    pub dst: Tile,
    pub special: Option<SpecialMove>,
}

impl FromStr for ChessMove {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 4 {
            return Err("Move string too short".to_string());
        }
        let src = Tile::from(&s[0..2]);
        let dst = Tile::from(&s[2..4]);
        let special = if s.len() > 4 {
            match &s[4..5] {
                "Q" | "q" => Some(SpecialMove::Queen),
                "R" | "r" => Some(SpecialMove::Rook),
                "B" | "b" => Some(SpecialMove::Bishop),
                "N" | "n" => Some(SpecialMove::Knight),
                _ => None,
            }
        } else {
            None
        };
        Ok(ChessMove { src, dst, special })
    }
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.src, self.dst)?;
        if let Some(special) = self.special {
            let c = match special {
                SpecialMove::Queen => 'Q',
                SpecialMove::Knight => 'N',
                SpecialMove::Rook => 'R',
                SpecialMove::Bishop => 'B',
                _ => ' ',
            };
            if c != ' ' {
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}

pub trait ToChessMove {
    fn parse(&self) -> Option<ChessMove>;
}

impl ToChessMove for String {
    fn parse(&self) -> Option<ChessMove> {
        log::trace!(
            "converting {}{}{}{}{} to chess move",
            fg_blue,
            style_bold,
            &self,
            fg_reset,
            style_reset
        );

        if !(4..=5).contains(&self.len()) {
            log::warn!("could not parse chess move: {}", self);
            return None;
        }

        let mut iter = self.chars();
        let src = Tile::new(iter.next()?, iter.next()?).or_else(|| {
            log::warn!("could not parse chess move: {}", self);
            None
        })?;
        let dst = Tile::new(iter.next()?, iter.next()?).or_else(|| {
            log::warn!("could not parse chess move: {}", self);
            None
        })?;

        let special = match iter.next() {
            Some('Q') => Some(SpecialMove::Queen),
            Some('N') => Some(SpecialMove::Knight),
            Some('R') => Some(SpecialMove::Rook),
            Some('B') => Some(SpecialMove::Bishop),
            _ => None,
        };

        Some(ChessMove { src, dst, special })
    }
}
