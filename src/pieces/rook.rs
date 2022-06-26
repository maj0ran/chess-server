pub struct Rook {
    color: Color,
    name: PieceType,
}

impl Rook {
    pub fn new(color: Color) -> Self {
        Rook {
            color,
            name: PieceType::Rook,
        }
    }
}

impl ChessPiece for Rook {
    fn get_moves<S: Into<String>>(&self, board: &Board, pos: S) -> Vec<String> {
        let pos = pos.into();
        let mut all_moves = vec![];
        let mut prev = pos.clone();
        while prev.up().rank() <= '8' {
            let next = prev.up();
            match board.peek(&next) {
                None => {
                    all_moves.push(next.clone());
                    prev = next;
                }
                Some(Piece { color, .. }) => {
                    if color != self.color {
                        all_moves.push(next.clone());
                    }
                    break;
                }
            };
        }
        prev = pos.clone();
        while prev.down().rank() >= '1' {
            let next = prev.down();
            match board.peek(&next) {
                None => {
                    all_moves.push(next.clone());
                    prev = next;
                }
                Some(Piece { color, .. }) => {
                    if color != self.color {
                        all_moves.push(next.clone());
                    }
                    break;
                }
            };
        }
        prev = pos.clone();
        while prev.right().file() <= 'h' {
            let next = prev.right();
            match board.peek(&next) {
                None => {
                    all_moves.push(next.clone());
                    prev = next;
                }
                Some(Piece { color, .. }) => {
                    if color != self.color {
                        all_moves.push(next.clone());
                    }
                    break;
                }
            };
        }
        prev = pos.clone();
        while prev.left().file() >= 'a' {
            let next = prev.left();
            match board.peek(&next) {
                None => {
                    all_moves.push(next.clone());
                    prev = next;
                }
                Some(Piece { color, .. }) => {
                    if color != self.color {
                        all_moves.push(next.clone());
                    }
                    break;
                }
            };
        }

        all_moves
    }
}
