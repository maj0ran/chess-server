use crate::field::Field;
use crate::piece;
use crate::pieces::*;
use core::fmt;

use crate::color::Color;
use crate::pieces::bishop::move_rules_bishop;
use crate::pieces::king::move_rules_king;
use crate::pieces::knight::move_rules_knight;
use crate::pieces::pawn::move_rules_pawn;
use crate::pieces::queen::move_rules_queen;
use crate::pieces::rook::move_rules_rook;

#[allow(dead_code)]
pub struct Board {
    pub fields: [Option<Piece>; 64],
    active_player: Color,
    castle_rights: [bool; 4],  // [K, Q, k, q]
    en_passant: Option<usize>, // index of field in linear memory
    half_moves: usize,
    full_moves: usize,
}

impl Board {
    pub fn new() -> Board {
        Board {
            fields: [None; 64],
            active_player: Color::White,
            castle_rights: [false; 4],
            en_passant: None,
            half_moves: 0,
            full_moves: 0,
        }
    }

    pub fn index(field: Field) -> usize {
        let file = field.file as u8 - 96;
        let rank = field.rank as u8 - 48;
        let rank = (8 - rank) + 1;
        let idx: usize = (((rank - 1) * 8) + (file - 1)) as usize;
        idx
    }

    pub fn peek(&self, idx: Field) -> &Option<Piece> {
        &self.fields[Board::index(idx)]
    }

    pub fn take(&mut self, idx: Field) -> Option<Piece> {
        self.fields[Board::index(idx)].take()
    }

    pub fn set(&mut self, idx: Field, piece: Option<Piece>) {
        self.fields[Board::index(idx)] = piece;
    }

    pub fn load_fen(fen: &str) -> Board {
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
        let mut fields = [None; 64];
        for c in iter {
            if c.is_alphabetic() {
                fields[curr_pos] = Some(piece!(c));
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
            _ => Some(Board::index(en_passant_str.to_string().into())),
        };

        // haf and full move
        let half_moves = usize::from_str_radix(half_move_str, 10).unwrap();
        let full_moves = usize::from_str_radix(full_move_str, 10).unwrap();

        Board {
            fields,
            active_player,
            castle_rights,
            en_passant,
            half_moves,
            full_moves,
        }
    }

    pub fn is_valid(&mut self, src: Field, dst: Field) -> bool {
        let p = self.peek(src);
        let valid_fields = match p {
            None => vec![],
            Some(p) => p.get_legal_fields(self, &src),
        };

        println!("{:?}", valid_fields);
        valid_fields.contains(&dst)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_string = String::new();

        // a .. h row
        board_string += " ";
        for i in 'a'..='h' {
            board_string = board_string + " " + i.to_string().as_str() + " ";
        }

        let mut rank_line = 8;
        for (i, piece) in self.fields.iter().enumerate() {
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
        board_string = board_string + format!("{}", self.active_player).as_str() + " ";
        board_string = board_string + format!("{}", self.half_moves).as_str() + " ";
        board_string = board_string + format!("{}", self.full_moves).as_str() + " ";
        write!(f, "{}", board_string)
    }
}
