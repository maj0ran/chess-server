use crate::piece;
use crate::pieces::*;
use crate::tile::Tile;
use core::fmt;
use std::ops::Index;
use std::ops::IndexMut;

use crate::color::Color;
use crate::pieces::bishop::Bishop;
use crate::pieces::king::King;
use crate::pieces::knight::Knight;
use crate::pieces::pawn::Pawn;
use crate::pieces::queen::Queen;
use crate::pieces::rook::Rook;

#[allow(dead_code)]
pub struct Game {
    pub tiles: [Option<Box<dyn PieceTrait + Send>>; 64],
    active_player: Color,
    castle_rights: [bool; 4], // [K, Q, k, q]
    en_passant: Option<Tile>, //_index of field in linear memory
    half_moves: usize,
    full_moves: usize,
}

impl Game {
    pub fn new() -> Game {
        Game::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq f3 0 1")
    }
    pub fn peek(&self, idx: Tile) -> &Option<Box<dyn PieceTrait + Send>> {
        &self[idx]
    }

    pub fn take(&mut self, idx: Tile) -> Option<Box<dyn PieceTrait + Send>> {
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

    pub fn load_fen(fen: &str) -> Game {
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
        let mut tiles: [Option<Box<dyn PieceTrait + Send>>; 64] = std::array::from_fn(|_| None);
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

        Game {
            tiles,
            active_player,
            castle_rights,
            en_passant,
            half_moves,
            full_moves,
        }
    }
    pub fn get_moves(&self, tile: Tile) -> Vec<Tile> {
        match &self[tile] {
            Some(p) => {
                if p.color() == self.active_player {
                    p.get_moves(self, tile)
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

    pub fn make_move(&mut self, src: Tile, dst: Tile) -> bool {
        if !self.is_valid(src, dst) {
            return false;
        } else {
            self[dst] = self.take(src);
            self.active_player = !self.active_player;
            return true;
        }
    }
}

impl Index<Tile> for Game {
    type Output = Option<Box<dyn PieceTrait + Send>>;

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

impl IndexMut<Tile> for Game {
    fn index_mut(&mut self, index: Tile) -> &mut Self::Output {
        let file: isize = index.file as isize - 96;
        let rank: isize = index.rank as isize - 48;
        let rank: isize = (8 - rank) + 1;
        let idx: isize = ((rank - 1) * 8) + (file - 1);
        &mut self.tiles[idx as usize]
    }
}
impl fmt::Display for Game {
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
