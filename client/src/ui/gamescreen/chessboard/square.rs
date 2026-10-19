use bevy::{
    color::Color,
    math::Vec2,
    picking::Pickable,
    prelude::*
};

use super::SQUARE_SIZE;

/// A single `ChessSquare` that can be rendered onto the board; preferably 64 of them.
/// A ChessSquare is bundled with:
/// - `Sprite`: its color (can be a texture too)
/// - `Transform`: size; for when the board is resized
/// - `Pickable`: Because the user can click chess squares.
#[derive(Component, Clone, Debug)]
#[require(Sprite, Transform, Pickable)]
pub struct ChessSquare {
    pub name: String,
    pub color: Color,
}

impl ChessSquare {
    pub(super) fn new(
        name: String,
        pos: Vec2,
        size: Vec2,
        color: Color,
    ) -> (ChessSquare, Sprite, Transform) {
        (
            ChessSquare { name, color },
            Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            Transform {
                translation: pos.extend(1.0),
                ..default()
            },
        )
    }
}
