use super::SQUARE_SIZE;
use bevy::color::Color;
use bevy::math::Vec2;
use bevy::picking::Pickable;
use bevy::prelude::*;

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
        color: Color,
    ) -> (ChessSquare, Sprite, Transform, Pickable) {
        (
            ChessSquare { name, color },
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
