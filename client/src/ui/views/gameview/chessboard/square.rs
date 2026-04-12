use super::SQUARE_SIZE;
use bevy::color::Color;
use bevy::math::Vec2;
use bevy::picking::Pickable;
use bevy::prelude::*;

/// 64 `ChessSquare`s are rendered onto the chessboard and are used to position pieces.
#[derive(Component, Clone, Debug)]
pub struct ChessSquare {
    pub x: u8,
    pub y: u8,
    pub name: String,
    pub color: Color,
}

impl ChessSquare {
    pub fn new(
        name: String,
        pos: Vec2,
        color: Color,
    ) -> (ChessSquare, Sprite, Transform, Pickable) {
        (
            ChessSquare {
                name,
                color,
                x: pos.x as u8,
                y: pos.y as u8,
            },
            Sprite {
                color,
                custom_size: Some(Vec2::splat(SQUARE_SIZE)),
                ..default()
            },
            Transform {
                translation: pos.extend(1.0),
                ..default()
            },
            Pickable::default(),
        )
    }
}
