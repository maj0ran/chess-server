use crate::{tile::Tile, util::*};

#[derive(Debug)]
pub enum SpecialMove {
    QueenPromotion,
    KnightPromotion,
    RookPromotion,
    BishopPromotion,
    KingsideCastle,
    QueensideCastle,
}

#[derive(Debug)]
pub struct ChessMove {
    pub src: Tile,
    pub dst: Tile,
    pub special: Option<SpecialMove>,
}

pub trait ToChessMove {
    fn parse(&self) -> Option<ChessMove>;
}

impl ToChessMove for String {
    fn parse(&self) -> Option<ChessMove> {
        log::trace!(
            "converting {fg_blue}{style_bold}{}{fg_reset}{style_reset} to chess move",
            &self
        );

        // chess moves are 4 or 5 chars long (d2d4 or b7b8Q)
        if self.len() > 5 || self.len() < 4 {
            log::warn!("could not parse chess move: {}", self);
            return None;
        }

        let mut iter = self.chars();
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();
        let src = Tile::new(file, rank);
        let src = match src {
            Some(t) => t,
            None => {
                log::warn!("could not parse chess move: {}", self);
                return None;
            }
        };
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();
        let dst = Tile::new(file, rank);
        let dst = match dst {
            Some(t) => t,
            None => {
                log::warn!("could not parse chess move: {}", self);
                return None;
            }
        };

        let special = match iter.next() {
            Some(p) => match p {
                'Q' => Some(SpecialMove::QueenPromotion),
                'N' => Some(SpecialMove::KnightPromotion),
                'R' => Some(SpecialMove::RookPromotion),
                'B' => Some(SpecialMove::BishopPromotion),
                _ => None,
            },
            None => None,
        };

        Some(ChessMove { src, dst, special })
    }
}
