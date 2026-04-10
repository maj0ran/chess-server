use crate::tile::Tile;
use crate::*;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Promotion {
    Queen,
    Knight,
    Rook,
    Bishop,
}

impl fmt::Display for Promotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Promotion::Queen => 'Q',
            Promotion::Knight => 'N',
            Promotion::Rook => 'R',
            Promotion::Bishop => 'B',
        };
        write!(f, "{}", c)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChessMove {
    pub src: Tile,
    pub dst: Tile,
    pub special: Option<Promotion>,
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
                "Q" | "q" => Some(Promotion::Queen),
                "R" | "r" => Some(Promotion::Rook),
                "B" | "b" => Some(Promotion::Bishop),
                "N" | "n" => Some(Promotion::Knight),
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
                Promotion::Queen => 'Q',
                Promotion::Knight => 'N',
                Promotion::Rook => 'R',
                Promotion::Bishop => 'B',
            };
            write!(f, "{}", c)?;
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
            Some('Q') => Some(Promotion::Queen),
            Some('N') => Some(Promotion::Knight),
            Some('R') => Some(Promotion::Rook),
            Some('B') => Some(Promotion::Bishop),
            _ => None,
        };

        Some(ChessMove { src, dst, special })
    }
}
