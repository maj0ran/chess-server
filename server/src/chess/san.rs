use crate::chess::chess::Chess;
use chess_core::{ChessColor, ChessMove, ChessPiece, Promotion, Tile};

pub trait San {
    fn to_san(&self, board: &Chess) -> String;
    #[allow(dead_code)]
    fn from_san(board: &Chess, san: &str) -> Option<ChessMove>;
}

impl San for ChessMove {
    /// Convert a `ChessMove` to Standard Algebraic Notation (SAN).
    /// This is only used to send the client the SAN representation of a move, so it
    /// can be displayed to the user. Internally, the server only works with `ChessMove`s.
    fn to_san(&self, board: &Chess) -> String {
        // First, we need the piece that has been moved
        let piece = match board.peek(self.src) {
            Some(p) => p,
            None => return String::new(),
        };

        // Detect castling by move coordinates
        if piece.typ == ChessPiece::King {
            if self.src == "e1" && self.dst == "g1" || self.src == "e8" && self.dst == "g8" {
                return "O-O".to_string();
            }
            if self.src == "e1" && self.dst == "c1" || self.src == "e8" && self.dst == "c8" {
                return "O-O-O".to_string();
            }
        }

        let mut san = String::new();
        // First char is piece-type, except for pawns
        if piece.typ != ChessPiece::Pawn {
            san.push(match piece.typ {
                ChessPiece::Knight => 'N',
                ChessPiece::Bishop => 'B',
                ChessPiece::Rook => 'R',
                ChessPiece::Queen => 'Q',
                ChessPiece::King => 'K',
                _ => unreachable!(), // pawn
            });

            // Disambiguation for pieces (not pawns)
            let mut candidates = Vec::new();
            // We check for every square if there are pieces that could also have made that move
            // TODO: inefficient? Maybe we could just trace back from dst square?
            for rank in '1'..='8' {
                for file in 'a'..='h' {
                    let src = Tile::new(file, rank).unwrap();
                    if src == self.src {
                        continue;
                    }
                    // check for every piece we find if it is the same wood and if it
                    // can also reach the destination square. If so, we save its tile
                    // for disambiguation.
                    if let Some(p) = board.peek(src) {
                        if p.typ == piece.typ && p.color == piece.color {
                            let moves = board.get_moves(src);
                            if moves.contains(&self.dst) {
                                candidates.push(src);
                            }
                        }
                    }
                }
            }

            // we have pieces that could also have made that move
            if !candidates.is_empty() {
                // we have a candidate, but they are on the same file, so adding the file isn't helping
                let file_ambiguous = candidates.iter().any(|c| c.file == self.src.file);
                // we have a candidate, but they are on the same rank, so adding the rank isn't helping
                let rank_ambiguous = candidates.iter().any(|c| c.rank == self.src.rank);

                if !file_ambiguous {
                    san.push(self.src.file);
                } else if !rank_ambiguous {
                    san.push(self.src.rank);
                } else {
                    san.push(self.src.file);
                    san.push(self.src.rank);
                }
            }
            // ... else Pawn
        } else {
            // Pawn capture: src file + 'x'
            if board.peek(self.dst).is_some() || board.en_passant == Some(self.dst) {
                san.push(self.src.file);
                san.push('x');
            }
        }

        // append dst file and rank
        san.push(self.dst.file);
        san.push(self.dst.rank);

        // append promotion
        if let Some(special) = self.special {
            match special {
                Promotion::Queen => san.push_str("=Q"),
                Promotion::Rook => san.push_str("=R"),
                Promotion::Bishop => san.push_str("=B"),
                Promotion::Knight => san.push_str("=N"),
            }
        }

        // Check/mate suffixes
        // for these suffixes we have to simulate the move on a cloned board and use its
        // functionality to test for checks and mates.
        let mut next_board = board.clone();
        let _ = next_board.make_move_unchecked(*self);
        // swap player since make_move_unchecked doesn't do it,
        // and we need the other player to test for check
        next_board.active_player = !next_board.active_player;
        let opponent = next_board.active_player;

        if next_board.is_checkmate() {
            san.push('#');
        } else if next_board.is_in_check(opponent) {
            san.push('+');
        }

        san
    }

    /// Converting a SAN to an internal `ChessMove`.
    /// HINT: This is not used for now, as client and server usually work with `ChessMove`s,
    /// and SAN is only used for humans who want to read the chess move. But let's keep it in case
    /// we want direct client<->server communication using SAN, e.g., when we use a UCI adapter.
    #[allow(dead_code)]
    fn from_san(board: &Chess, san: &str) -> Option<ChessMove> {
        let san = san.trim();
        // check castle moves first
        if san == "O-O" {
            return resolve_castle(board, true);
        } else if san == "O-O-O" {
            return resolve_castle(board, false);
        }

        let mut piece_type = ChessPiece::Pawn;
        let target;
        let mut disambiguation_file = None;
        let mut disambiguation_rank = None;
        let mut promotion = None;

        let mut chars: Vec<char> = san.chars().collect();

        // Remove check/checkmate suffixes
        if chars.last() == Some(&'+') || chars.last() == Some(&'#') {
            chars.pop();
        }

        if chars.is_empty() {
            return None;
        }

        // Promotion at the end
        if chars.len() >= 2 && chars[chars.len() - 2] == '=' {
            let p = chars.pop().unwrap();
            chars.pop(); // '='
            promotion = match p {
                'N' => Some(ChessPiece::Knight),
                'B' => Some(ChessPiece::Bishop),
                'R' => Some(ChessPiece::Rook),
                'Q' => Some(ChessPiece::Queen),
                _ => return None,
            };
        } else if chars.len() >= 3 {
            // FIDE also allows =less promotion: e8Q
            // Check if it's a pawn move to 8th/1st rank with a piece suffix
            let last = *chars.last().unwrap();
            if ['N', 'B', 'R', 'Q'].contains(&last) {
                // Peek at what would be the rank
                let rank_idx = chars.len() - 2;
                let rank = chars[rank_idx];
                if rank == '8' || rank == '1' {
                    let p = chars.pop().unwrap();
                    promotion = match p {
                        'N' => Some(ChessPiece::Knight),
                        'B' => Some(ChessPiece::Bishop),
                        'R' => Some(ChessPiece::Rook),
                        'Q' => Some(ChessPiece::Queen),
                        _ => unreachable!(),
                    };
                }
            }
        }

        // Piece type
        if let Some(&p) = chars.first() {
            match p {
                'N' => {
                    piece_type = ChessPiece::Knight;
                    chars.remove(0);
                }
                'B' => {
                    piece_type = ChessPiece::Bishop;
                    chars.remove(0);
                }
                'R' => {
                    piece_type = ChessPiece::Rook;
                    chars.remove(0);
                }
                'Q' => {
                    piece_type = ChessPiece::Queen;
                    chars.remove(0);
                }
                'K' => {
                    piece_type = ChessPiece::King;
                    chars.remove(0);
                }
                'a'..='h' => {
                    piece_type = ChessPiece::Pawn;
                }
                _ => return None,
            }
        }

        // Target square must be present.
        // It's the last 2 characters of what's left.
        if chars.len() < 2 {
            return None;
        }

        let rank = chars.pop().unwrap();
        let file = chars.pop().unwrap();
        if !('1'..='8').contains(&rank) || !('a'..='h').contains(&file) {
            return None;
        }
        target = Tile::new(file, rank).unwrap();

        // Remaining characters are disambiguation and capture
        if !chars.is_empty() && chars.last() == Some(&'x') {
            chars.pop();
        }

        for &c in &chars {
            if ('a'..='h').contains(&c) {
                disambiguation_file = Some(c);
            } else if ('1'..='8').contains(&c) {
                disambiguation_rank = Some(c);
            } else {
                return None;
            }
        }

        resolve_move(
            board,
            piece_type,
            target,
            disambiguation_file,
            disambiguation_rank,
            promotion,
        )
    }
}

fn resolve_castle(board: &Chess, kingside: bool) -> Option<ChessMove> {
    let color = board.active_player;
    let rank = if color == ChessColor::White { '1' } else { '8' };
    let src = Tile::new('e', rank).unwrap();
    let dst_file = if kingside { 'g' } else { 'c' };
    let dst = Tile::new(dst_file, rank).unwrap();
    let special = None;

    if let Some(p) = board.peek(src) {
        if p.typ == ChessPiece::King && p.color == color {
            let moves = board.get_moves(src);
            if moves.contains(&dst) {
                return Some(ChessMove { src, dst, special });
            }
        }
    }
    None
}

fn resolve_move(
    board: &Chess,
    piece_type: ChessPiece,
    dst: Tile,
    dis_file: Option<char>,
    dis_rank: Option<char>,
    promotion: Option<ChessPiece>,
) -> Option<ChessMove> {
    let color = board.active_player;
    let mut candidates = Vec::new();

    for rank in '1'..='8' {
        for file in 'a'..='h' {
            let src = Tile::new(file, rank).unwrap();
            if let Some(p) = board.peek(src) {
                if p.typ == piece_type && p.color == color {
                    if let Some(df) = dis_file {
                        if src.file != df {
                            continue;
                        }
                    }
                    if let Some(dr) = dis_rank {
                        if src.rank != dr {
                            continue;
                        }
                    }

                    let moves = board.get_moves(src);
                    if moves.contains(&dst) {
                        candidates.push(src);
                    }
                }
            }
        }
    }

    if candidates.len() == 1 {
        let src = candidates[0];
        let special = promotion.map(|p| match p {
            ChessPiece::Queen => Promotion::Queen,
            ChessPiece::Rook => Promotion::Rook,
            ChessPiece::Bishop => Promotion::Bishop,
            ChessPiece::Knight => Promotion::Knight,
            _ => Promotion::Queen,
        });
        Some(ChessMove { src, dst, special })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_san_to_move() {
        let game = Chess::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let res = ChessMove::from_san(&game, "e4");
        assert_eq!(res.map(|m| m.to_string()), Some("e2e4".to_string()));
    }

    #[test]
    fn test_san_from_move_pawn() {
        let game = Chess::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let mv = ChessMove::from_str("e2e4").unwrap();
        assert_eq!(mv.to_san(&game), "e4");

        let game = Chess::load_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2");
        let mv = ChessMove::from_str("e4d5").unwrap();
        assert_eq!(mv.to_san(&game), "exd5");
    }

    #[test]
    fn test_san_from_move_piece() {
        let game = Chess::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let mv = ChessMove::from_str("g1f3").unwrap();
        assert_eq!(mv.to_san(&game), "Nf3");
    }

    #[test]
    fn test_san_from_move_disambiguation() {
        let game =
            Chess::load_fen("r1bqkbnr/pppp1ppp/2n5/4p3/4P3/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 3");
        // White knights at c3 and g1 can both reach e2
        let mv1 = ChessMove::from_str("g1e2").unwrap();
        assert_eq!(mv1.to_san(&game), "Nge2");

        let mv2 = ChessMove::from_str("c3e2").unwrap();
        assert_eq!(mv2.to_san(&game), "Nce2");

        let game2 = Chess::load_fen("rnbqkbnr/8/8/8/8/8/8/R1R1K2R w KQkq - 0 1");
        // Rooks at a1 and c1 can both reach b1
        let mv3 = ChessMove::from_str("a1b1").unwrap();
        assert_eq!(mv3.to_san(&game2), "Rab1");
    }

    #[test]
    fn test_san_from_move_double_disambiguation() {
        let game = Chess::load_fen("8/7k/8/8/Q1Q5/Q7/8/7K w - - 2 2");
        let mv = ChessMove::from_str("a4b3").unwrap();

        assert_eq!(mv.to_san(&game), "Qa4b3");
    }

    #[test]
    fn test_san_from_move_castling() {
        let game =
            Chess::load_fen("rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 5");
        let mv = ChessMove {
            src: Tile::new('e', '1').unwrap(),
            dst: Tile::new('g', '1').unwrap(),
            special: None,
        };
        assert_eq!(mv.to_san(&game), "O-O");
    }

    #[test]
    fn test_san_from_move_promotion() {
        let game = Chess::load_fen("8/4P3/8/8/8/8/8/k6K w - - 0 1");
        let mv = ChessMove {
            src: Tile::new('e', '7').unwrap(),
            dst: Tile::new('e', '8').unwrap(),
            special: Some(Promotion::Queen),
        };
        assert_eq!(mv.to_san(&game), "e8=Q");
    }

    #[test]
    fn test_san_from_move_check() {
        let game = Chess::load_fen("rnbqkbnr/ppppp1pp/8/5p2/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2");
        let mv = ChessMove::from_str("d1h5").unwrap();
        assert_eq!(mv.to_san(&game), "Qh5+");
    }

    #[test]
    fn test_san_to_move_extended() {
        let game = Chess::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let res = ChessMove::from_san(&game, "e4");
        assert_eq!(res.map(|m| m.to_string()), Some("e2e4".to_string()));

        let res = ChessMove::from_san(&game, "Nf3");
        assert_eq!(res.map(|m| m.to_string()), Some("g1f3".to_string()));
    }
}
