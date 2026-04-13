use crate::chess::chess::Chess;
use crate::chess::pieces::Piece;
use crate::chess::san::San;
use chess_core::piece::Id;
use chess_core::{ChessColor, ChessMove, ChessPiece, Tile, WoodPiece};
use rand::Rng;

#[derive(Clone)]
pub struct ZobristHash {
    is_black: u64,
    table: [[u64; 12]; 64],
    castle_rights: [u64; 4],
    en_passant_file: [u64; 8],

    hash: u64,
}

impl ZobristHash {
    pub fn new() -> Self {
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

            hash: 0,
        }
    }

    pub fn update_hash(
        &mut self,
        tiles: Vec<(Tile, Option<Piece>, Option<Piece>)>, // (Tile, old_piece, new_piece)
        old_castle_rights: [bool; 4],
        new_castle_rights: [bool; 4],
        old_en_passant: Option<Tile>,
        new_en_passant: Option<Tile>,
        turn_changed: bool,
    ) {
        if turn_changed {
            self.hash ^= self.is_black;
        }

        for i in 0..4 {
            if old_castle_rights[i] != new_castle_rights[i] {
                self.hash ^= self.castle_rights[i];
            }
        }

        if let Some(t) = old_en_passant {
            let f = t.file as u8 - 97;
            self.hash ^= self.en_passant_file[f as usize];
        }

        if let Some(t) = new_en_passant {
            let f = t.file as u8 - 97;
            self.hash ^= self.en_passant_file[f as usize];
        }

        for (tile, old_piece, new_piece) in tiles {
            let idx = tile.to_index();
            if let Some(p) = old_piece {
                self.hash ^= self.table[idx as usize][p.piece.id() as usize];
            }
            if let Some(p) = new_piece {
                self.hash ^= self.table[idx as usize][p.piece.id() as usize];
            }
        }
    }

    pub fn set_hash(
        &mut self,
        tiles: &[Option<Piece>; 64],
        active_player: ChessColor,
        castle_rights: &[bool; 4],
        en_passant: Option<Tile>,
    ) {
        self.hash = self.get_hash(tiles, active_player, castle_rights, en_passant);
    }

    pub fn get_current_hash(&self) -> u64 {
        self.hash
    }

    pub fn get_hash(
        &self,
        tiles: &[Option<Piece>; 64],
        active_player: ChessColor,
        castle_rights: &[bool; 4],
        en_passant: Option<Tile>,
    ) -> u64 {
        let mut hash = 0;
        if active_player == ChessColor::Black {
            hash = hash ^ self.is_black;
        }
        for (i, &rights) in castle_rights.iter().enumerate() {
            hash = hash ^ (self.castle_rights[i] * if rights { 1 } else { 0 });
        }
        if let Some(t) = en_passant {
            let f = t.file as u8 - 97;
            hash = hash ^ self.en_passant_file[f as usize];
        };

        for (i, t) in tiles.iter().enumerate() {
            if let Some(p) = t {
                hash = hash ^ self.table[i][p.piece.id() as usize];
            }
        }

        hash
    }
}

#[test]
fn test_hash_init() {
    let chess = Chess::load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let start_pos = chess.hash.get_current_hash();

    let mut chess = chess;
    let _ = chess.make_move(ChessMove::from_san(&chess, "Nc3").unwrap());
    let _ = chess.make_move(ChessMove::from_san(&chess, "Nc6").unwrap());
    let _ = chess.make_move(ChessMove::from_san(&chess, "Nb1").unwrap());
    let _ = chess.make_move(ChessMove::from_san(&chess, "Nb8").unwrap());
    let back_to_start_pos = chess.hash.get_current_hash();

    assert_eq!(
        start_pos, back_to_start_pos,
        "Hash should be the same after Nc3Nc6Nb1Nb8"
    );
}

#[test]
fn test_incremental_hash() {
    let mut chess = Chess::new();
    let moves = ["e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Ba4", "Nf6", "O-O"];

    for m in moves {
        let mov = ChessMove::from_san(&chess, m).unwrap();
        chess.make_move(mov).unwrap();

        let full_hash = chess.hash.get_hash(
            &chess.tiles,
            chess.active_player,
            &chess.castle_rights,
            chess.en_passant,
        );
        assert_eq!(
            chess.hash.get_current_hash(),
            full_hash,
            "Incremental hash should match full hash after move {}",
            m
        );
    }
}
