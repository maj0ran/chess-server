use super::assets::ChessAssets;
use bevy::prelude::*;

/// A `ChessPiece` is the visual representation of a chess piece on the board.
/// It is bundled with a `Sprite` (the texture) and a `Transform` (size, rotation).
///
/// The rotation is a bit wacky: Because the chessboard is rotated 180 degrees
/// for the black player, the pieces are flipped, too. Thus, we have to flip them back
/// again to have a board upside down but with pieces on their feet.
///
/// `id` is the character representation of the piece, e.g. 'B' for a white bishop
/// or 'n' for a black knight.
#[derive(Component, Debug)]
#[require(Sprite, Transform)]
pub struct ChessPiece {
    pub id: char,
}

impl ChessPiece {
    pub(super) fn new(
        id: char,
        assets: &ChessAssets,
        flip: bool,
    ) -> (ChessPiece, Sprite, Transform) {
        let image = assets.get(id).expect("missing chess piece asset");
        (
            ChessPiece { id },
            Sprite {
                image: image.clone(),
                ..default()
            },
            Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                rotation: Quat::from_rotation_z(if flip { std::f32::consts::PI } else { 0.0 }),
                ..default()
            },
        )
    }
}
