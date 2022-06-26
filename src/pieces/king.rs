struct King {
    color: Color,
    name: PieceType,
}

impl ChessPiece for King {
    fn get_moves<S: Into<String>>(&self, board: &Board, pos: S) -> Vec<String> {
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
}
