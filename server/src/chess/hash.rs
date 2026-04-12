use crate::chess::chess::Chess;
use crate::chess::san::San;
use chess_core::{ChessColor, ChessMove, ChessPiece, Tile, WoodPiece};
use rand::Rng;

trait Id {
    fn id(&self) -> u32;
}

impl Id for WoodPiece {
    fn id(&self) -> u32 {
        match self {
            WoodPiece {
                typ: ChessPiece::King,
                color: ChessColor::White,
            } => 0,
            WoodPiece {
                typ: ChessPiece::Queen,
                color: ChessColor::White,
            } => 1,
            WoodPiece {
                typ: ChessPiece::Rook,
                color: ChessColor::White,
            } => 2,
            WoodPiece {
                typ: ChessPiece::Bishop,
                color: ChessColor::White,
            } => 3,
            WoodPiece {
                typ: ChessPiece::Knight,
                color: ChessColor::White,
            } => 4,
            WoodPiece {
                typ: ChessPiece::Pawn,
                color: ChessColor::White,
            } => 5,
            WoodPiece {
                typ: ChessPiece::King,
                color: ChessColor::Black,
            } => 6,
            WoodPiece {
                typ: ChessPiece::Queen,
                color: ChessColor::Black,
            } => 7,
            WoodPiece {
                typ: ChessPiece::Rook,
                color: ChessColor::Black,
            } => 8,
            WoodPiece {
                typ: ChessPiece::Bishop,
                color: ChessColor::Black,
            } => 9,
            WoodPiece {
                typ: ChessPiece::Knight,
                color: ChessColor::Black,
            } => 10,
            WoodPiece {
                typ: ChessPiece::Pawn,
                color: ChessColor::Black,
            } => 11,
        }
    }
}

struct ZobristHash {
    is_black: u64,
    table: [[u64; 12]; 64],
    castle_rights: [u64; 4],
    en_passant_file: [u64; 8],
}

impl ZobristHash {
    fn new() -> Self {
        let mut rng = rand::rng();

        let is_black = rng.next_u64();

        let mut table = [[0; 12]; 64];
        for i in 0..64 {
            for j in 0..12 {
                table[i][j] = rng.next_u64();
            }
        }

        let mut castle_rights = [0; 4];
        for i in 0..4 {
            castle_rights[i] = rng.next_u64();
        }

        let mut en_passant_file = [0; 8];
        for i in 0..8 {
            en_passant_file[i] = rng.next_u64();
        }

        Self {
            is_black,
            table,
            castle_rights,
            en_passant_file,
        }
    }

    fn get_hash(&self, chess: &Chess) -> u64 {
        let mut hash = 0;
        if chess.active_player == ChessColor::Black {
            hash = hash ^ self.is_black;
        }
        for (i, &rights) in chess.castle_rights.iter().enumerate() {
            hash = hash ^ (self.castle_rights[i] * if rights { 1 } else { 0 });
        }
        if let Some(t) = chess.en_passant {
            let f = t.file as u8 - 97;
            hash = hash ^ self.en_passant_file[f as usize];
        };

        let board = chess.tiles;
        for (i, t) in board.iter().enumerate() {
            if let Some(p) = t {
                hash = hash ^ self.table[i][p.piece.id() as usize];
            }
        }

        hash
    }
}

#[test]
fn test_hash_init() {
    let zh = ZobristHash::new();
    let mut chess = Chess::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let start_pos = zh.get_hash(&chess);
    chess.make_move(ChessMove::from_san(&chess, "Nc3").unwrap());
    chess.make_move(ChessMove::from_san(&chess, "Nc6").unwrap());
    chess.make_move(ChessMove::from_san(&chess, "Nb1").unwrap());
    chess.make_move(ChessMove::from_san(&chess, "Nb8").unwrap());
    let back_to_start_pos = zh.get_hash(&chess);

    assert_eq!(
        start_pos, back_to_start_pos,
        "Hash should be the same after Nc3Nc6Nb1Nb8"
    );
}
