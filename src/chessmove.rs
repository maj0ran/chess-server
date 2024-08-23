use log::debug;

use crate::{pieces::ChessPiece, tile::Tile, util::*};

pub struct ChessMove {
    pub src: Tile,
    pub dst: Tile,
    pub promotion: Option<ChessPiece>,
}

pub trait ToChessMove {
    fn parse(&self) -> Option<ChessMove>;
}

impl ToChessMove for String {
    fn parse(&self) -> Option<ChessMove> {
        debug!(
            "converting {fg_blue}{style_bold}{}{fg_reset}{style_reset} to chess move",
            &self
        );

        let mut iter = self.chars();
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();
        let src = Tile::new(file, rank);
        let src = match src {
            Some(t) => t,
            None => return None,
        };
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();
        let dst = Tile::new(file, rank);
        let dst = match dst {
            Some(t) => t,
            None => return None,
        };

        let promotion = match iter.next() {
            Some(p) => match p {
                'Q' => Some(ChessPiece::Queen),
                'N' => Some(ChessPiece::Knight),
                'R' => Some(ChessPiece::Rook),
                'B' => Some(ChessPiece::Bishop),
                _ => None,
            },
            None => None,
        };

        Some(ChessMove {
            src,
            dst,
            promotion,
        })
    }
}
