use log::debug;
use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;

use crate::chess::pieces::*;
use crate::piece;
use chess_core::*;

/// The chess struct holds all information for a game of chess.
/// This is basically the same information also encoded in a FEN.
/// Everything addendum, like real players, material count, etc. is wrapped around this struct.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Chess {
    pub tiles: [Option<Piece>; 64],
    pub active_player: ChessColor,
    pub castle_rights: [bool; 4], // [K, Q, k, q]
    pub en_passant: Option<Tile>,
    half_moves: usize,
    full_moves: usize,
}

/// makes it possible to iterate over a chessboard.
/// iterating over a chesboard means going through all tiles of the board.
pub struct ChessboardIterator<'a> {
    board: &'a Chess,
    pub index: usize,
}

impl<'a> Iterator for ChessboardIterator<'a> {
    type Item = Option<Piece>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.board.tiles.len() {
            return None;
        }
        let tile = Some(self.board.tiles[self.index]);
        self.index += 1;
        tile
    }
}

impl<'a> IntoIterator for &'a Chess {
    type Item = Option<Piece>;
    type IntoIter = ChessboardIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ChessboardIterator {
            board: self,
            index: 0,
        }
    }
}

impl Chess {
    /// create a new chess board with default starting position.
    pub fn new() -> Chess {
        Chess::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    /// peek onto a tile which piece is on it. Doesn't change the tile.
    pub fn peek(&self, idx: Tile) -> &Option<Piece> {
        &self[idx]
    }

    /// take the piece from a tile, making it empty.
    pub fn take(&mut self, idx: Tile) -> Option<Piece> {
        self[idx].take()
    }

    /// determine if the given tile can be attacked by any piece from the given player.
    pub fn is_attacked(&self, tile: Tile, by_player: ChessColor) -> bool {
        let mut iter = Tile::all().into_iter();
        while let Some(t) = iter.next() {
            match self[t] {
                Some(piece) => {
                    if piece.color == by_player {
                        let attacking_tiles = self.get_tiles_controlled(t);
                        if attacking_tiles.contains(&tile) {
                            return true;
                        }
                    }
                }
                None => {}
            }
        }
        false
    }

    /// get the position of the king of the given player.
    pub fn get_king_pos(&self, color: ChessColor) -> Tile {
        for t in Tile::all() {
            if let Some(p) = self[t] {
                if p.typ == ChessPiece::King && p.color == color {
                    return t;
                }
            }
        }
        /* TODO: this should rather return Option<Tile> because a panic! crashes the whole server.
        even though this should never happen, it kinda feels awkward. */
        panic!("No king found for color {:?}", color);
    }

    /// determine if the king of that player is in check.
    pub fn is_in_check(&self, player: ChessColor) -> bool {
        let king_pos = self.get_king_pos(player);
        self.is_attacked(king_pos, !player)
    }

    /// shoots a ray from a tile. returns all tiles on this ray until a piece is hit,
    /// INCLUDING the tile with the hitten piece
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

    /// construct a game from a FEN string
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
                let p = piece!(c);
                tiles[curr_pos] = p;
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
            ChessColor::White
        } else {
            ChessColor::Black
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
            _ => Some(Tile::from(en_passant_str)),
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

    /// Get the FEN of the current position.
    pub fn get_fen(&self) -> String {
        let mut fen = String::with_capacity(90);

        // 1. Piece placement
        for rank in 0..8 {
            let mut empty_count = 0;
            for file in 0..8 {
                let idx = rank * 8 + file;
                if let Some(p) = self.tiles[idx] {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    fen.push(p.as_byte());
                } else {
                    empty_count += 1;
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if rank < 7 {
                fen.push('/');
            }
        }

        // 2. Active player
        fen.push(' ');
        match self.active_player {
            ChessColor::White => fen.push('w'),
            ChessColor::Black => fen.push('b'),
        }

        // 3. Castling rights
        fen.push(' ');
        let mut any_castle = false;
        if self.castle_rights[0] {
            fen.push('K');
            any_castle = true;
        }
        if self.castle_rights[1] {
            fen.push('Q');
            any_castle = true;
        }
        if self.castle_rights[2] {
            fen.push('k');
            any_castle = true;
        }
        if self.castle_rights[3] {
            fen.push('q');
            any_castle = true;
        }
        if !any_castle {
            fen.push('-');
        }

        // 4. En passant
        fen.push(' ');
        if let Some(t) = self.en_passant {
            fen.push_str(&t.to_string());
        } else {
            fen.push('-');
        }

        // 5. Half-move clock
        fen.push(' ');
        fen.push_str(&self.half_moves.to_string());

        // 6. Full-move counter
        fen.push(' ');
        fen.push_str(&self.full_moves.to_string());

        fen
    }

    ///get the tiles controlled by the piece on the given tile
    ///A controlled tile is a tile that is attacked by a piece, e.g., a empty tile or a tile with an
    ///opponent piece. For pawns, the diagonal tiles are attacking tiles.
    pub fn get_tiles_controlled(&self, tile: Tile) -> Vec<Tile> {
        match &self[tile] {
            Some(p) => (p.get_tiles_controlled)(self, tile),
            None => vec![],
        }
    }

    /// get all tiles the piece on the given tile can move to.
    /// these can be different from the controlled tiles, e.g., for pawns
    pub fn get_moves(&self, tile: Tile) -> Vec<Tile> {
        match &self[tile] {
            Some(p) => (p.get_moves)(self, tile),
            None => vec![],
        }
    }

    /// Get all moves a player can make. Is used for determining checkmate and stalemate.
    pub fn get_all_moves_for_player(&self, player: ChessColor) -> Vec<Tile> {
        let mut valid_moves = vec![];
        for tile in Tile::all() {
            let piece = self[tile];
            if let Some(p) = piece {
                if p.color != player {
                    continue;
                } else {
                    for m in self.get_moves(tile) {
                        let cm = ChessMove {
                            src: tile,
                            dst: m,
                            special: None,
                        };

                        let mut simulation = self.clone();
                        if let Ok(_) = simulation.make_move_unchecked(cm) {
                            if !simulation.is_in_check(simulation.active_player) {
                                valid_moves.push(m);
                            }
                        }
                    }
                }
            }
        }
        valid_moves
    }

    /// True if the current active player is checkmated.
    pub fn is_checkmate(&self) -> bool {
        self.get_all_moves_for_player(self.active_player).is_empty()
            && self.is_in_check(self.active_player)
    }

    /// True if stalemate.
    pub fn is_stalemate(&self) -> bool {
        self.get_all_moves_for_player(self.active_player).is_empty()
            && !self.is_in_check(self.active_player)
    }

    /// Helper method to handle en passant moves.
    /// updates the vector of moves for this piece.
    /// Only makes sense to use with pawns.
    fn handle_en_passant(
        &mut self,
        piece: &Piece,
        dst: Tile,
        updated_tiles: &mut Vec<(Tile, Option<Piece>)>,
    ) {
        if piece.typ == ChessPiece::Pawn {
            if let Some(en_passant_tile) = self.en_passant {
                if dst == en_passant_tile {
                    let capture_dir = if piece.color == ChessColor::White {
                        Tile::DOWN
                    } else {
                        Tile::UP
                    };
                    if let Some(capture_tile) = dst + capture_dir {
                        self.take(capture_tile);
                        updated_tiles.push((capture_tile, None));
                    }
                }
            }
        }
    }

    /// Helper method to handle castling moves.
    /// Updates the vector of moves for this piece.
    /// Only makes sense to use with kings.
    fn handle_castling(
        &mut self,
        piece: &Piece,
        src: Tile,
        dst: Tile,
        updated_tiles: &mut Vec<(Tile, Option<Piece>)>,
    ) {
        if piece.typ == ChessPiece::King {
            let (rank, home_rank) = if piece.color == ChessColor::White {
                ('1', '1')
            } else {
                ('8', '8')
            };

            if src.file == 'e' && src.rank == home_rank && dst.rank == rank {
                if dst.file == 'g' {
                    let rook_src = Tile::new('h', rank).unwrap();
                    let rook_dst = Tile::new('f', rank).unwrap();
                    self[rook_dst] = self.take(rook_src);
                    updated_tiles.push((rook_src, None));
                    updated_tiles.push((rook_dst, self[rook_dst]));
                } else if dst.file == 'c' {
                    let rook_src = Tile::new('a', rank).unwrap();
                    let rook_dst = Tile::new('d', rank).unwrap();
                    self[rook_dst] = self.take(rook_src);
                    updated_tiles.push((rook_src, None));
                    updated_tiles.push((rook_dst, self[rook_dst]));
                }
            }
        }
    }

    /// Helper method to handle promotion moves.
    /// Only makes sense to use it with pawns.
    fn handle_promotion(&self, piece: &mut Piece, dst: Tile, special: &Option<Promotion>) {
        if let Some(promotion) = special {
            if piece.typ == ChessPiece::Pawn && (dst.rank == '8' || dst.rank == '1') {
                let c = match promotion {
                    Promotion::Queen => 'Q',
                    Promotion::Knight => 'N',
                    Promotion::Rook => 'R',
                    Promotion::Bishop => 'B',
                };

                let c = if piece.color == ChessColor::White {
                    c
                } else {
                    c.to_lowercase().next().unwrap()
                };

                *piece = piece!(c).unwrap();
            }
        }
    }

    /// Helper method to update castling rights.
    /// If king or a rook moves, update the loss of castling rights.
    fn update_castle_rights(&mut self, piece: &Piece, src: Tile) {
        // if King moves, lose both castling rights
        if piece.typ == ChessPiece::King {
            if piece.color == ChessColor::White {
                self.castle_rights[0] = false;
                self.castle_rights[1] = false;
            } else {
                self.castle_rights[2] = false;
                self.castle_rights[3] = false;
            }
        }

        if piece.typ == ChessPiece::Rook {
            // king-side rook moves
            if src.file == 'h' {
                if piece.color == ChessColor::White {
                    self.castle_rights[0] = false;
                } else {
                    self.castle_rights[2] = false;
                }
            }
            // queen-side rook moves
            if src.file == 'a' {
                if piece.color == ChessColor::White {
                    self.castle_rights[1] = false;
                } else {
                    self.castle_rights[3] = false;
                }
            }
        }
    }

    /// Helper method to update en passant square.
    /// If pawn moves two squares, update the en passant square.
    fn update_en_passant_square(&mut self, piece: &Piece, src: Tile, dst: Tile) {
        if piece.typ == ChessPiece::Pawn {
            self.en_passant = if piece.color == ChessColor::White {
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
        } else {
            self.en_passant = None;
        }
    }

    /// Execute a move on the board.
    /// Returns a vector of tiles that have been changed.
    /// This approach is helpful for en passant and castling, where more tiles
    /// than the src and dest tiles are affected.
    pub fn make_move(&mut self, mov: ChessMove) -> ChessResult<Vec<(Tile, Option<Piece>)>> {
        // 1. Check movement rules
        let src = mov.src;
        let dst = mov.dst;
        let p = self.peek(src).ok_or(ChessError::IllegalMove(mov))?;

        if p.color != self.active_player {
            return Err(ChessError::IllegalMove(mov));
        }

        let tiles = self.get_moves(src);
        if !tiles.contains(&dst) {
            return Err(ChessError::IllegalMove(mov));
        }

        if p.typ == ChessPiece::Pawn && (dst.rank == '8' || dst.rank == '1') {
            if mov.special.is_none() {
                return Err(ChessError::IllegalMove(mov));
            }
        }

        // 2. Simulate the move on a cloned board
        let mut simulation = self.clone();

        // 3. Call make_move_unchecked on simulation. This handles all the movement rules.
        let _ = simulation.make_move_unchecked(mov)?;

        // 4. Call is_valid_position on simulation to check if the move is legal regarding checks.
        if simulation.is_in_check(simulation.active_player) {
            return Err(ChessError::IllegalMove(mov));
        }

        // 5. Special check for castling through check
        // A bit awkward to make the test here, but it's yet again such a special chess rule
        // castling is the only move we can't do to leave a check + we can't castle through check.
        if p.typ == ChessPiece::King && (mov.dst.file as i8 - mov.src.file as i8).abs() == 2 {
            if self.is_in_check(self.active_player) {
                return Err(ChessError::IllegalMove(mov));
            }
            let direction = if mov.dst.file == 'g' { 1 } else { -1 };
            let through_file = (mov.src.file as i8 + direction) as u8 as char;
            let through_tile = Tile::new(through_file, mov.src.rank).unwrap();
            if self.is_attacked(through_tile, !self.active_player) {
                return Err(ChessError::IllegalMove(mov));
            }
        }

        // 6. If all succeeded, we do the real move on the real board.
        // but first, we need to get the information we need to update the half-moves field
        // for the 50-moves-rules.
        let is_capture = self[dst].is_some();
        let is_pawn_move = self[src].unwrap().typ == ChessPiece::Pawn;

        let updated_tiles = self.make_move_unchecked(mov)?;

        // update full moves and half moves
        if self.active_player == ChessColor::White {
            self.full_moves += 1;
        }
        if is_capture || is_pawn_move {
            self.half_moves = 0;
        } else {
            self.half_moves += 1;
        }

        // it's the opponents turn now
        self.active_player = !self.active_player;

        debug!("executed move: {style_bold}{fg_green}{src}{dst}{style_reset}{fg_reset}!");
        Ok(updated_tiles)
    }

    /// Helper method for pseudo-legal moves. These are all moves that can be made by the
    /// movement rules of the pieces but where checks are not considered.
    pub fn make_move_unchecked(
        &mut self,
        chessmove: ChessMove,
    ) -> ChessResult<Vec<(Tile, Option<Piece>)>> {
        let src = chessmove.src;
        let dst = chessmove.dst;

        let mut piece = self.take(src).ok_or(ChessError::IllegalMove(chessmove))?;

        if piece.color != self.active_player {
            self[src] = Some(piece); // restore piece
            return Err(ChessError::IllegalMove(chessmove));
        }

        let mut updated_tiles = vec![(src, None)];
        // handle special cases. note that en_passant and castling need to update different tiles
        // than the destination tile
        self.handle_en_passant(&piece, dst, &mut updated_tiles);
        self.handle_castling(&piece, src, dst, &mut updated_tiles);
        self.handle_promotion(&mut piece, dst, &chessmove.special);

        /* now the destination tile gets updated with our moved piece */
        self[dst] = Some(piece);
        updated_tiles.push((dst, Some(piece)));

        self.update_castle_rights(&piece, src);
        self.update_en_passant_square(&piece, src, dst);

        Ok(updated_tiles)
    }

    pub fn is_fifty_moves_rule(&self) -> bool {
        self.full_moves >= 100
    }
}

/// Indexing Tiles on a Chess board.
/// index[0] is a8, index[63] is h1
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

impl Index<&str> for Chess {
    type Output = Option<Piece>;

    fn index(&self, index: &str) -> &Self::Output {
        &self[Tile::from(index)]
    }
}

impl IndexMut<&str> for Chess {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        &mut self[Tile::from(index)]
    }
}

/// using print!() on a chessboard will return a formatted CLI board.
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

/// couple of tests.
/// TODO: These are quite unsorted and were written spontaneously during development.
/// TODO: Some kind of test module would be nice.
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tile_index_alignment() {
        let board = Chess::new();
        // a8 should be at index 0
        assert_eq!(Tile::from("a8").to_index(), 0);
        // h1 should be at index 63
        assert_eq!(Tile::from("h1").to_index(), 63);

        // Check piece at a8 (Rook)
        assert_eq!(board.tiles[0].unwrap().typ, ChessPiece::Rook);
        assert_eq!(board.tiles[0].unwrap().color, ChessColor::Black);

        // Check if index access matches
        assert_eq!(board[Tile::from("a8")].unwrap().typ, ChessPiece::Rook);
    }

    #[test]
    fn test_check_detection() {
        // Scholar's mate style check
        let game = Chess::load_fen("rnbqkbnr/ppppp1pp/8/5p1Q/4P3/8/PPPP1PPP/RNB1KBNR b KQkq - 0 1");
        assert!(game.is_in_check(ChessColor::Black));
        assert!(!game.is_in_check(ChessColor::White));
    }

    #[test]
    fn test_invalid_move_revealing_check() {
        // White Rook e1, Black Bishop e7, Black King e8. Bishop moves to d6.
        let game = Chess::load_fen("4k3/4b3/8/8/8/8/4R3/4K3 b - - 0 1");
        let mv: ChessMove = "e7d6".parse().unwrap();
        let mut simulation = game.clone();
        simulation.make_move_unchecked(mv).unwrap();
        assert!(simulation.is_in_check(simulation.active_player)); // Reveals check
    }

    #[test]
    fn test_must_move_out_of_check() {
        // Scholar's mate style check
        let mut game = Chess::new(); // Starts with White turn
        game.make_move("e2e4".parse().unwrap()).unwrap();
        game.make_move("f7f5".parse().unwrap()).unwrap();
        game.make_move("d1h5".parse().unwrap()).unwrap();

        assert!(game.is_in_check(ChessColor::Black));

        // Illegal move: a7a6
        let mv: ChessMove = "a7a6".parse().unwrap();
        assert!(game.make_move(mv).is_err());

        // Legal move: g7g6 (blocks check)
        let mv: ChessMove = "g7g6".parse().unwrap();
        assert!(game.make_move(mv).is_ok());
    }

    #[test]
    fn test_castling_restrictions_with_check() {
        // Setup: White king e1, Rook h1.
        let mut game = Chess::load_fen("4k3/4r3/8/8/8/8/8/4K2R w K - 0 1");
        // White to move. King at e1, Rook at h1. Black Rook at e7.
        assert!(game.is_in_check(ChessColor::White));
        assert!(game
            .make_move("e1g1".parse::<ChessMove>().unwrap())
            .is_err());
    }
}
