pub mod board;

use crate::game::board::Board;
use crate::game::board::Color;
use crate::game::board::PieceType;
use crate::game::board::Position;
use crate::piece;
use crate::pos;
use crate::Piece;

pub struct GameState {}

pub struct Game {
    pub board: Board,
}

impl Game {
    pub fn new() -> Game {
        let board = Board::new();

        let mut game = Game { board };

        game.setup();
        game
    }

    pub fn setup(&mut self) {
        self.board.setup();
        self.spawn(piece!("K"), Some(pos!("e1")));
        self.spawn(piece!("k"), Some(pos!("e8")));

        self.spawn(piece!("Q"), Some(pos!("d1")));
        self.spawn(piece!("q"), Some(pos!("d8")));

        self.spawn(piece!("B"), Some(pos!("c1")));
        self.spawn(piece!("b"), Some(pos!("c8")));
        self.spawn(piece!("B"), Some(pos!("f1")));
        self.spawn(piece!("b"), Some(pos!("f8")));

        self.spawn(piece!("N"), Some(pos!("b1")));
        self.spawn(piece!("n"), Some(pos!("b8")));
        self.spawn(piece!("N"), Some(pos!("g1")));
        self.spawn(piece!("n"), Some(pos!("g8")));

        self.spawn(piece!("R"), Some(pos!("a1")));
        self.spawn(piece!("r"), Some(pos!("a8")));
        self.spawn(piece!("R"), Some(pos!("h1")));
        self.spawn(piece!("r"), Some(pos!("h8")));

        self.spawn(piece!("P"), Some(pos!("a2")));
        self.spawn(piece!("P"), Some(pos!("b2")));
        self.spawn(piece!("P"), Some(pos!("c2")));
        self.spawn(piece!("P"), Some(pos!("d2")));
        self.spawn(piece!("P"), Some(pos!("e2")));
        self.spawn(piece!("P"), Some(pos!("f2")));
        self.spawn(piece!("P"), Some(pos!("g2")));
        self.spawn(piece!("P"), Some(pos!("h2")));

        self.spawn(piece!("p"), Some(pos!("a7")));
        self.spawn(piece!("p"), Some(pos!("b7")));
        self.spawn(piece!("p"), Some(pos!("c7")));
        self.spawn(piece!("p"), Some(pos!("d7")));
        self.spawn(piece!("p"), Some(pos!("e7")));
        self.spawn(piece!("p"), Some(pos!("f7")));
        self.spawn(piece!("p"), Some(pos!("g7")));
        self.spawn(piece!("p"), Some(pos!("h7")));
    }

    pub fn spawn(&mut self, piece: Piece, pos: Option<Position>) {
        match pos {
            Some(e) => self.board.put(e, piece),
            None => todo!(),
        }
    }

    pub fn select(&mut self, pos: Position) -> bool {
        self.board.select(pos)
    }

    pub fn get_valid_moves(&self) -> Vec<Position> {
        self.board
            .get_valid_moves(self.board.selected_field.unwrap())
    }

    pub fn move_to(&mut self, src: Position, dst: Position) -> bool {
        let result = self.board.try_move(src, dst);
        if result {
            self.board.active_player = match self.board.active_player {
                Color::White => Color::Black,
                Color::Black => Color::White,
            };
            true
        } else {
            false
        }
    }
}
