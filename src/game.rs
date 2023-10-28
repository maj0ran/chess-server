use crate::util::*;
use log::info;

use crate::piece;
use crate::pieces::*;
use crate::tile::Tile;
use core::fmt;
use std::ops::Index;
use std::ops::IndexMut;

use crate::color::Color;
use crate::pieces::bishop::*;
use crate::pieces::king::*;
use crate::pieces::knight::*;
use crate::pieces::pawn::*;
use crate::pieces::queen::*;
use crate::pieces::rook::*;

#[allow(dead_code)]
pub struct Chess {
    pub tiles: [Option<Piece>; 64],
    active_player: Color,
    pub castle_rights: [bool; 4], // [K, Q, k, q]
    pub en_passant: Option<Tile>,
    half_moves: usize,
    full_moves: usize,
}

impl Chess {
    pub fn new() -> Chess {
        Chess::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq f3 0 1")
    }
    pub fn peek(&self, idx: Tile) -> &Option<Piece> {
        &self[idx]
    }

    pub fn take(&mut self, idx: Tile) -> Option<Piece> {
        self[idx].take()
    }

    pub fn ray(&self, src: Tile, dir: (i8, i8)) -> Vec<Tile> {
        let mut tiles = Vec::<Tile>::new();
        let mut d = src + dir;
        while let Some(t) = d {
            tiles.push(t);
            if let Some(_) = self.peek(t) {
                break;
            }
            d = t + dir;
        }
        tiles
    }

    pub fn load_fen(fen: &str) -> Chess {
        let mut curr_pos = 0;
        let mut fen_iter = fen.split(" ");
        let pos_str = fen_iter.next().unwrap();
        let player_str = fen_iter.next().unwrap();
        let castle_str = fen_iter.next().unwrap();
        let en_passant_str = fen_iter.next().unwrap();
        let half_move_str = fen_iter.next().unwrap();
        let full_move_str = fen_iter.next().unwrap();

        // iterate through position string
        let iter = pos_str.chars();
        let mut tiles: [Option<Piece>; 64] = std::array::from_fn(|_| None);
        for c in iter {
            if c.is_alphabetic() {
                tiles[curr_pos] = Some(piece!(c));
                curr_pos += 1;
            } else if c.is_numeric() {
                curr_pos += char::to_digit(c, 10).unwrap() as usize;
            } else if c == '/' {
                assert!(curr_pos % 8 == 0)
            }
        }

        // rest of the string for game state
        // player next to move
        let mut iter = player_str.chars();
        let active_player = iter.next();
        assert!(active_player == Some('b') || active_player == Some('w'));
        let active_player = if active_player == Some('w') {
            Color::White
        } else {
            Color::Black
        };

        // castling rights
        let mut castle_rights = [false; 4];
        let iter = castle_str.chars();
        for c in iter {
            match c {
                'K' => castle_rights[0] = true,
                'Q' => castle_rights[1] = true,
                'k' => castle_rights[2] = true,
                'q' => castle_rights[3] = true,
                _ => {}
            }
        }

        // en passant field
        let en_passant = match en_passant_str {
            "-" => None,
            _ => Some(Tile::from(en_passant_str.to_string())),
        };

        // haf and full move
        let half_moves = usize::from_str_radix(half_move_str, 10).unwrap();
        let full_moves = usize::from_str_radix(full_move_str, 10).unwrap();

        Chess {
            tiles,
            active_player,
            castle_rights,
            en_passant,
            half_moves,
            full_moves,
        }
    }

    pub fn get_fen(&self) {
        let mut fen = String::new();
        for (i, t) in self.tiles.iter().enumerate() {
            if i % 8 == 0 {
                fen = fen + "/";
            }
            if let Some(p) = t {
                //     fen = fen + p.to_fen().to_string().as_str();
            }
        }
    }
    pub fn get_moves(&self, tile: Tile) -> Vec<Tile> {
        match &self[tile] {
            Some(p) => {
                if p.color == self.active_player {
                    (p.get_moves)(self, tile)
                } else {
                    vec![]
                }
            }
            None => vec![],
        }
    }
    fn is_valid(&self, src: Tile, dst: Tile) -> bool {
        let p = self.peek(src);
        let tiles = match p {
            None => vec![],
            Some(_) => self.get_moves(src),
        };

        tiles.contains(&dst)
    }

    // This method returns a List of all tiles that has updated.
    // this approach is helpful for en passant and castling.
    pub fn make_move(&mut self, src: Tile, dst: Tile) -> Vec<(Tile, Option<Piece>)> {
        let mut tiles: Vec<(Tile, Option<Piece>)> = Vec::new();
        // check if the move is valid
        if !self.is_valid(src, dst) {
            return vec![];
        }

        let mut piece = self.take(src).unwrap(); // cannot fail
        piece.move_count += 1;
        let piece = piece; // de-mut, because I don't trust myself
        tiles.push((src, None)); // we move a piece, so the source tile gets empty

        // special rule for en passant
        if self.en_passant.is_some() {
            if dst == self.en_passant.unwrap() {
                if self.active_player == Color::White {
                    let _ = self.take((dst + Tile::DOWN).unwrap());
                    tiles.push(((dst + Tile::DOWN).unwrap(), None));
                } else {
                    let _ = self.take((dst + Tile::UP).unwrap());
                    tiles.push(((dst + Tile::UP).unwrap(), None));
                };
            }
        }
        tiles.push((dst, Some(piece)));
        self[dst] = Some(piece);

        let piece = self[dst].as_ref().unwrap(); // this can never fail
        if piece.typ == ChessPiece::Pawn {
            let en_passant_tile = if piece.color == Color::White {
                if src.rank == '2' && dst.rank == '4' {
                    dst + Tile::DOWN
                } else {
                    None
                }
            } else {
                // Black
                if src.rank == '7' && dst.rank == '5' {
                    dst + Tile::UP
                } else {
                    None
                }
            };
            if en_passant_tile.is_some() {
                info!(
                    "en_passant: {style_bold}{fg_magenta}{}{fg_reset}{style_reset}",
                    en_passant_tile.unwrap()
                );
            }
            self.en_passant = en_passant_tile;
        };
        self.active_player = !self.active_player;
        tiles
    }
}

// index[0] is a8, index[63] is h1
impl Index<Tile> for Chess {
    type Output = Option<Piece>;

    fn index(&self, index: Tile) -> &Self::Output {
        let file: isize = index.file as isize - 96;
        let rank: isize = index.rank as isize - 48;
        let rank: isize = (8 - rank) + 1;
        let idx: isize = ((rank - 1) * 8) + (file - 1);
        let idx = idx as usize;
        if idx > 63 {
            return &None;
        }
        &self.tiles[idx as usize]
    }
}

impl IndexMut<Tile> for Chess {
    fn index_mut(&mut self, index: Tile) -> &mut Self::Output {
        let file: isize = index.file as isize - 96;
        let rank: isize = index.rank as isize - 48;
        let rank: isize = (8 - rank) + 1;
        let idx: isize = ((rank - 1) * 8) + (file - 1);
        &mut self.tiles[idx as usize]
    }
}
impl fmt::Display for Chess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_string = String::new();

        // a .. h row
        board_string += " ";
        for i in 'a'..='h' {
            board_string = board_string + " " + i.to_string().as_str() + " ";
        }

        let mut rank_line = 8;
        for (i, piece) in self.tiles.iter().enumerate() {
            if i % 8 == 0 {
                board_string = board_string + "\n\n" + rank_line.to_string().as_str();
                rank_line -= 1;
            }
            let p = &piece;
            match p {
                Some(piece) => {
                    board_string = board_string + " " + format!("{}", piece).as_str() + " "
                }

                None => board_string = board_string + "   ",
            }
        }
        board_string += "\n";
        board_string += "               "; // 15 spaces to right-align text under board
        board_string = board_string + format!("{}", self.active_player).as_str() + " ";
        board_string = board_string + format!("{}", self.half_moves).as_str() + " ";
        board_string = board_string + format!("{}", self.full_moves).as_str() + " ";
        write!(f, "{}", board_string)
    }
}
