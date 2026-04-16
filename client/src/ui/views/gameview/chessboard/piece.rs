use super::assets::ChessAssets;
use bevy::prelude::*;

/// A `ChessPiece` is attached to a `ChessSquare` and is used to render a piece on the board.
#[derive(Component, Debug)]
pub struct ChessPiece {
    pub id: char,
}

impl ChessPiece {
    pub fn new(
        id: char,
        assets: &Res<ChessAssets>,
        flip: bool,
    ) -> (ChessPiece, Sprite, Transform, Pickable) {
        let image = assets.get(id).unwrap();
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
            Pickable::default(),
        )
    }
}
