use core::fmt;
use std::error::Error;
use std::ops::{Index, IndexMut};

#[derive(Debug, Eq, PartialEq)]
pub enum Color {
    Black,
    White,
}

#[derive(Debug)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Debug)]
struct OutOfBoardError {
    details: Position,
}

impl OutOfBoardError {
    fn new(pos: Position) -> OutOfBoardError {
        OutOfBoardError { details: pos }
    }
}

#[macro_export]
macro_rules! in_board {
    ($p:ident) => {
        $p.rank >= '1' && $p.rank <= '8' && $p.file >= 'a' && $p.file <= 'h'
    };
}

/* draws a line from pos to direction. direction may be multiple
 * fields, like (up, left) for Bishop.
 * the line will either end at the edge of a board or in front
 *  of a friendly piece or at an enemy piece
*/
#[macro_export]
macro_rules! line {
    ($board:ident, $start:ident, $($dir:ident),*) => {{
        let mut legal_moves = Vec::new();
        let mut tmp = $start;
        loop {
        $(
            tmp = tmp.$dir();
        )*
            if in_board!(tmp) {
                match &$board[tmp].piece {
                    Some(p) => {

                                if(p.color != $board.active_player) {
                                    legal_moves.push(tmp);
                                    break;
                                } else {
                                    break;
                                }
                    },
                    None => { legal_moves.push(tmp)},
                }
            } else {
                break;
            }
        }
        legal_moves
    }};
}

#[macro_export]
macro_rules! pos {
    ($p:expr) => {{
        let mut it = $p.chars();
        let file = it.next();
        let rank = it.next();
        Position::new(file.unwrap(), rank.unwrap())
    }};
}

#[macro_export]
macro_rules! piece {
    ($p:expr) => {{
        let p = $p.chars().next();

        let p = match p {
            Some('K') => Piece::new(PieceType::King, Color::White),
            Some('k') => Piece::new(PieceType::King, Color::Black),
            Some('Q') => Piece::new(PieceType::Queen, Color::White),
            Some('q') => Piece::new(PieceType::Queen, Color::Black),
            Some('R') => Piece::new(PieceType::Rook, Color::White),
            Some('r') => Piece::new(PieceType::Rook, Color::Black),
            Some('N') => Piece::new(PieceType::Knight, Color::White),
            Some('n') => Piece::new(PieceType::Knight, Color::Black),
            Some('B') => Piece::new(PieceType::Bishop, Color::White),
            Some('b') => Piece::new(PieceType::Bishop, Color::Black),
            Some('P') => Piece::new(PieceType::Pawn, Color::White),
            Some('p') => Piece::new(PieceType::Pawn, Color::Black),
            _ => panic!(),
        };
        p
    }};
}

#[derive(Debug)]
pub struct Field {
    pos: Position,
    pub piece: Option<Piece>,
}

impl Field {
    pub fn new(pos: Position) -> Field {
        Field { pos, piece: None }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Position {
    file: char,
    rank: char,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.file, self.rank)
    }
}

impl Position {
    pub fn new(file: char, rank: char) -> Position {
        Position { file, rank }
    }

    pub fn validate(self) -> Option<Position> {
        match (self.rank >= '1' && self.rank <= '8' && self.file >= 'a' && self.file <= 'h') {
            true => Some(self),
            false => None,
        }
    }
    pub fn idx(field: &str) -> u8 {
        let mut it = field.chars();
        let file = it.next().unwrap();
        let rank = it.next().unwrap();

        let file = file as u8 - 96;
        let rank = rank as u8 - 48;

        ((rank - 1) * 8) + (file - 1)
    }

    fn right(&self) -> Position {
        let file = (self.file as u8 + 1) as char;
        let rank = self.rank;
        Position { file, rank }
    }

    fn left(&self) -> Position {
        let file = (self.file as u8 - 1) as char;
        let rank = self.rank;
        Position { file, rank }
    }

    fn up(&self) -> Position {
        let file = self.file;
        let rank = (self.rank as u8 + 1) as char;
        Position { file, rank }
    }

    fn down(&self) -> Position {
        let file = self.file;
        let rank = (self.rank as u8 - 1) as char;
        Position { file, rank }
    }
}
impl From<&str> for Position {
    fn from(item: &str) -> Self {
        let mut it = item.chars();
        let file = it.next().unwrap();
        let rank = it.next().unwrap();
        Position { file, rank }
    }
}
impl Into<String> for Position {
    fn into(self) -> String {
        let mut result = String::new();
        result.push(self.file);
        result.push(self.rank);
        result
    }
}

impl From<usize> for Position {
    fn from(item: usize) -> Self {
        let file: char = (item % 8 + 96 + 1) as u8 as char;
        let rank: char = (item / 8 + 48 + 1) as u8 as char;

        Position { file, rank }
    }
}

impl std::ops::AddAssign<Position> for Position {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            file: ((self.file) as u8 + (other.file) as u8) as char,
            rank: ((self.rank) as u8 + (other.rank) as u8) as char,
        };
    }
}

#[derive(Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
    pub value: u8,
    pub move_count: usize,
}
impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Piece {
        let value = match piece_type {
            PieceType::Pawn => 1,
            PieceType::Knight => 3,
            PieceType::Bishop => 3,
            PieceType::Rook => 5,
            PieceType::Queen => 9,
            PieceType::King => 255,
        };

        Piece {
            piece_type,
            value,
            color,
            move_count: 0,
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Piece {
                piece_type: PieceType::King,
                color: Color::Black,
                ..
            } => "♔".to_string(),
            Piece {
                piece_type: PieceType::Queen,
                color: Color::Black,
                ..
            } => "♕".to_string(),
            Piece {
                piece_type: PieceType::Rook,
                color: Color::Black,
                ..
            } => "♖".to_string(),
            Piece {
                piece_type: PieceType::Bishop,
                color: Color::Black,
                ..
            } => "♗".to_string(),
            Piece {
                piece_type: PieceType::Knight,
                color: Color::Black,
                ..
            } => "♘".to_string(),
            Piece {
                piece_type: PieceType::Pawn,
                color: Color::Black,
                ..
            } => "♙".to_string(),

            Piece {
                piece_type: PieceType::King,
                color: Color::White,
                ..
            } => "♚".to_string(),
            Piece {
                piece_type: PieceType::Queen,
                color: Color::White,
                ..
            } => "♛".to_string(),
            Piece {
                piece_type: PieceType::Rook,
                color: Color::White,
                ..
            } => "♜".to_string(),
            Piece {
                piece_type: PieceType::Bishop,
                color: Color::White,
                ..
            } => "♝".to_string(),
            Piece {
                piece_type: PieceType::Knight,
                color: Color::White,
                ..
            } => "♞".to_string(),
            Piece {
                piece_type: PieceType::Pawn,
                color: Color::White,
                ..
            } => "♟".to_string(),
        };

        write!(f, "{}", symbol)
    }
}

pub struct Board {
    pub active_player: Color,
    fields: Vec<Field>,
    pub selected_field: Option<Position>,
}
/*
 * creates a new 8x8 chess board with no pieces on it
 */
impl Board {
    pub fn new() -> Board {
        let mut fields = Vec::new();
        for i in 0..64 {
            let f = Field::new(Position::from(i));
            fields.push(Field::new(Position::from(i)));
        }

        Board {
            fields,
            active_player: Color::White,
            selected_field: None,
        }
    }

    pub fn select(&mut self, pos: Position) -> bool {
        let valid = match &self[pos].piece {
            Some(Piece { color, .. }) => {
                if *color == self.active_player {
                    true
                } else {
                    false
                }
            }
            None => false,
        };

        if valid {
            self.selected_field = Some(pos);
        }
        valid
    }

    fn get_jumps(&self, pos: Position) -> Vec<Position> {
        let mut legal_moves = Vec::new();
        let dsts = vec![
            pos.up().up().right().validate(),
            pos.up().up().left().validate(),
            pos.right().right().up().validate(),
            pos.right().right().down().validate(),
            pos.down().down().left().validate(),
            pos.down().down().right().validate(),
            pos.left().left().down().validate(),
            pos.left().left().up().validate(),
        ];

        let dsts = dsts.iter().filter(|x| x.is_some());
        for dst in dsts {
            let dst = dst.unwrap();
            match &self[dst].piece {
                Some(p) => match &p {
                    Piece { color, .. } => {
                        if *color != self.active_player {
                            legal_moves.push(dst);
                        }
                    }
                },
                None => legal_moves.push(dst),
            }
        }
        legal_moves
    }

    pub fn get_valid_moves(&self, pos: Position) -> Vec<Position> {
        let mut moves = Vec::new();
        let src = &self[pos];
        match &src.piece {
            Some(p) => match p {
                Piece {
                    piece_type: PieceType::Pawn,
                    color: Color::White,
                    ..
                } => {
                    let dsts = vec![pos.up().left(), pos.up().right()];
                    for dst in dsts {
                        if !in_board!(dst) {
                            continue;
                        }
                        match &self[dst].piece {
                            Some(Piece { color, .. }) => {
                                if *color == Color::Black {
                                    moves.push(dst);
                                }
                            }
                            None => {}
                        }
                    }
                    let dst = pos.up();
                    if !in_board!(dst) {
                        return moves;
                    }
                    if self[dst].piece.is_none() {
                        moves.push(dst);
                    }

                    let dst = pos.up().up();
                    if !in_board!(dst) {
                        return moves;
                    }
                    if self[dst].piece.is_none() {
                        if p.move_count == 0 {
                            moves.push(dst);
                        }
                    }
                }
                Piece {
                    piece_type: PieceType::Pawn,
                    color: Color::Black,
                    ..
                } => {
                    let dsts = vec![pos.down().left(), pos.down().right()];
                    for dst in dsts {
                        if !in_board!(dst) {
                            continue;
                        }
                        match &self[dst].piece {
                            Some(Piece { color, .. }) => {
                                if *color == Color::White {
                                    moves.push(dst);
                                }
                            }
                            None => {}
                        }
                    }
                    let dst = pos.down();
                    if !in_board!(dst) {
                        return moves;
                    }
                    if self[dst].piece.is_none() {
                        moves.push(dst);
                    }

                    let dst = pos.down().down();
                    if !in_board!(dst) {
                        return moves;
                    }
                    if self[dst].piece.is_none() {
                        if p.move_count == 0 {
                            moves.push(dst);
                        }
                    }
                }
                Piece {
                    piece_type: PieceType::Rook,
                    ..
                } => {
                    moves.append(&mut line!(self, pos, up));
                    moves.append(&mut line!(self, pos, down));
                    moves.append(&mut line!(self, pos, left));
                    moves.append(&mut line!(self, pos, right));
                }
                Piece {
                    piece_type: PieceType::Bishop,
                    ..
                } => {
                    moves.append(&mut line!(self, pos, up, right));
                    moves.append(&mut line!(self, pos, up, left));
                    moves.append(&mut line!(self, pos, down, right));
                    moves.append(&mut line!(self, pos, down, left));
                }
                Piece {
                    piece_type: PieceType::Knight,
                    ..
                } => moves.append(&mut self.get_jumps(pos)),
                Piece {
                    piece_type: PieceType::King,
                    ..
                } => {
                    let mut v = Vec::new();
                    v.push(pos.up());
                    v.push(pos.down());
                    v.push(pos.right());
                    v.push(pos.left());
                    v.push(pos.up().left());
                    v.push(pos.up().right());
                    v.push(pos.down().left());
                    v.push(pos.down().right());
                    v.retain(|&x| x.file >= 'a' && x.file <= 'h' && x.rank >= '1' && x.rank <= '8');
                    v.retain(|&x| self[x].piece.is_none());

                    moves.append(&mut v);
                }
                Piece {
                    piece_type: PieceType::Queen,
                    ..
                } => {
                    moves.append(&mut line!(self, pos, up));
                    moves.append(&mut line!(self, pos, down));
                    moves.append(&mut line!(self, pos, right));
                    moves.append(&mut line!(self, pos, left));
                    moves.append(&mut line!(self, pos, up, right));
                    moves.append(&mut line!(self, pos, up, left));
                    moves.append(&mut line!(self, pos, down, right));
                    moves.append(&mut line!(self, pos, down, left));
                }
            },
            None => {}
        }
        moves
    }
    /* check if the current positioning is a check ... */
    pub fn check_check(&mut self) -> bool {
        println!("Checking for Check...");
        let fields = &self.fields;
        let mut check = false;
        for field in fields {
            //            println!("Computing Field {}", field.pos);
            match &field.piece {
                None => {}
                Some(attacking) => {
                    for defending in self.get_valid_moves(field.pos) {
                        match &self[defending].piece {
                            None => {}
                            Some(p) => {
                                println!(
                                    "{} {} -> {} {}",
                                    attacking,
                                    field.pos,
                                    self[defending].piece.as_ref().unwrap(),
                                    defending
                                );
                            }
                            Some(Piece {
                                piece_type: PieceType::King,
                                color,
                                ..
                            }) => {
                                if *color != attacking.color {
                                    println!("+++CHECK+++");
                                    check = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            };
        }
        check
    }
    pub fn try_move(&mut self, src: Position, dst: Position) -> bool {
        match &self[src].piece {
            Some(p) => {
                let reachable_fields = self.get_valid_moves(src);
                for f in &reachable_fields {
                    println!("{} {} ", p, f)
                }
                if reachable_fields.iter().any(|&x| x == dst) {
                    self.move_to(src, dst);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn move_to(&mut self, src: Position, dst: Position) {
        self[dst].piece = self[src].piece.take();
        let p: &mut Piece = self[dst].piece.as_mut().unwrap();
        p.move_count += 1;
        if self.check_check() {
            println!("!!! CHECK !!!");
        }
    }

    pub fn put(&mut self, pos: Position, piece: Piece) {
        self[pos].piece = Some(piece);
    }

    pub fn setup(&mut self) -> &mut Self {
        self
    }
}
impl IndexMut<Position> for Board {
    fn index_mut<'a>(&'a mut self, pos: Position) -> &'a mut Field {
        let file = pos.file as u8 - 96;
        let rank = pos.rank as u8 - 48;

        let idx: usize = (((rank - 1) * 8) + (file - 1)) as usize;
        &mut self.fields[idx]
    }
}
impl std::ops::Index<Position> for Board {
    type Output = Field;
    fn index<'a>(&'a self, pos: Position) -> &'a Field {
        let file = pos.file as u8 - 96;
        let rank = pos.rank as u8 - 48;

        let idx: usize = (((rank - 1) * 8) + (file - 1)) as usize;
        &self.fields[idx]
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_string = String::new();
        board_string += " ";
        for i in 'a'..='h' {
            board_string = board_string + " " + i.to_string().as_str() + " ";
        }

        let mut rank_line = 1;
        for (i, field) in self.fields.iter().enumerate() {
            if i % 8 == 0 {
                board_string = board_string + "\n\n" + rank_line.to_string().as_str();
                rank_line += 1;
            }
            let p = &field.piece;
            match p {
                Some(piece) => {
                    board_string = board_string + " " + format!("{}", piece).as_str() + " "
                }

                None => board_string = board_string + "   ",
            }
        }
        write!(f, "{}", board_string)
    }
}
