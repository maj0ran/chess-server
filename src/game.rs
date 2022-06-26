use crate::pieces::*;
use core::fmt;

#[macro_export]
macro_rules! piece {
    ($p:expr) => {{
        let p = match $p {
            'K' => Piece::new(PieceType::King, Color::White),
            'k' => Piece::new(PieceType::King, Color::Black),
            'Q' => Piece::new(PieceType::Queen, Color::White),
            'q' => Piece::new(PieceType::Queen, Color::Black),
            'R' => Piece::new(PieceType::Rook, Color::White),
            'r' => Piece::new(PieceType::Rook, Color::Black),
            'N' => Piece::new(PieceType::Knight, Color::White),
            'n' => Piece::new(PieceType::Knight, Color::Black),
            'B' => Piece::new(PieceType::Bishop, Color::White),
            'b' => Piece::new(PieceType::Bishop, Color::Black),
            'P' => Piece::new(PieceType::Pawn, Color::White),
            'p' => Piece::new(PieceType::Pawn, Color::Black),
            _ => panic!(),
        };
        p
    }};
}

pub trait ChessField {
    fn up(&self) -> Self;
    fn down(&self) -> Self;
    fn left(&self) -> Self;
    fn right(&self) -> Self;
    fn file(&self) -> char;
    fn rank(&self) -> char;
}

impl ChessField for String {
    fn up(&self) -> Self {
        let mut iter = self.chars();
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();

        let rank = (rank as u8 + 1) as char;

        let mut result = file.to_string();
        result.push(rank);
        result
    }

    fn down(&self) -> Self {
        let mut iter = self.chars();
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();

        let rank = (rank as u8 - 1) as char;

        let mut result = file.to_string();
        result.push(rank);
        result
    }

    fn left(&self) -> Self {
        let mut iter = self.chars();
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();

        let file = (file as u8 - 1) as char;

        let mut result = file.to_string();
        result.push(rank);
        result
    }

    fn right(&self) -> Self {
        let mut iter = self.chars();
        let file = iter.next().unwrap();
        let rank = iter.next().unwrap();

        let file = (file as u8 + 1) as char;

        let mut result = file.to_string();
        result.push(rank);
        result
    }

    fn file(&self) -> char {
        self.chars().nth(0).unwrap()
    }

    fn rank(&self) -> char {
        self.chars().nth(1).unwrap()
    }
}

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
    #[allow(dead_code)]
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

    fn index<S: Into<String>>(field: S) -> usize {
        let field = field.into();
        let mut it = field.chars();
        let file = it.next().unwrap() as u8 - 96;
        let rank = it.next().unwrap() as u8 - 48;
        let rank = (8 - rank) + 1;
        let idx: usize = (((rank - 1) * 8) + (file - 1)) as usize;

        idx
    }

    pub fn peek<S: Into<String>>(&self, idx: S) -> Option<Piece> {
        self.fields[Board::index(idx)]
    }

    pub fn take<S: Into<String>>(&mut self, idx: S) -> Option<Piece> {
        self.fields[Board::index(idx)].take()
    }

    pub fn set<S: Into<String>>(&mut self, idx: S, piece: Option<Piece>) {
        self.fields[Board::index(idx)] = piece;
    }

    pub fn get_moves<S: Into<String>>(&mut self, idx: S) -> Vec<String> {
        let idx = idx.into();
        let p = self.peek(&idx);
        if p.is_none() {
            return Vec::new();
        }
        let p = p.unwrap(); // cannot fail because of above early return
        let x = match p {
            Piece {
                piece_type: PieceType::King,
                ..
            } => {
                let mut all_moves = vec![
                    idx.up(),
                    idx.down(),
                    idx.left(),
                    idx.right(),
                    idx.up().left(),
                    idx.up().right(),
                    idx.down().left(),
                    idx.down().right(),
                ];

                all_moves.retain(|m| Board::index(m) > 0 && Board::index(m) < 64);
                let moves: Vec<&String> = all_moves
                    .iter()
                    .filter(|m| match self.peek(*m) {
                        None => true,
                        Some(Piece { color, .. }) => color != p.color,
                    })
                    .collect();

                let mut pseudolegal_moves = Vec::<String>::new();
                for m in moves {
                    pseudolegal_moves.push(String::from(m.clone()));
                }
                pseudolegal_moves
            }
            Piece {
                piece_type: PieceType::Rook,
                ..
            } => {
                let mut all_moves = vec![];
                let mut prev = idx.clone();
                while prev.up().rank() <= '8' {
                    let next = prev.up();
                    match self.peek(&next) {
                        None => {
                            all_moves.push(next.clone());
                            prev = next;
                        }
                        Some(Piece { color, .. }) => {
                            if color != p.color {
                                all_moves.push(next.clone());
                            }
                            break;
                        }
                    };
                }
                prev = idx.clone();
                while prev.down().rank() >= '1' {
                    let next = prev.down();
                    match self.peek(&next) {
                        None => {
                            all_moves.push(next.clone());
                            prev = next;
                        }
                        Some(Piece { color, .. }) => {
                            if color != p.color {
                                all_moves.push(next.clone());
                            }
                            break;
                        }
                    };
                }
                prev = idx.clone();
                while prev.right().file() <= 'h' {
                    let next = prev.right();
                    match self.peek(&next) {
                        None => {
                            all_moves.push(next.clone());
                            prev = next;
                        }
                        Some(Piece { color, .. }) => {
                            if color != p.color {
                                all_moves.push(next.clone());
                            }
                            break;
                        }
                    };
                }
                prev = idx.clone();
                while prev.left().file() >= 'a' {
                    let next = prev.left();
                    match self.peek(&next) {
                        None => {
                            all_moves.push(next.clone());
                            prev = next;
                        }
                        Some(Piece { color, .. }) => {
                            if color != p.color {
                                all_moves.push(next.clone());
                            }
                            break;
                        }
                    };
                }

                all_moves
            }
            _ => {
                println!("Not Implemented");
                let mut v = Vec::new();
                v.push(String::from("Not Implemented"));
                v
            }
        };

        x
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
            _ => Some(Board::index(en_passant_str)),
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

    pub fn move_to<S: Into<String>>(&mut self, src: S, dst: S) {
        let p = self.take(src);
        self.set(dst, p);
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
