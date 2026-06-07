use crate::backend::ActiveGame;
use crate::ui::theme::RIGHT_PANEL_RATIO;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;

#[derive(Event)]
pub struct MoveHistoryUpdated;
#[derive(Event)]
pub struct MoveHistoryFullRefresh;

#[derive(Component, Clone, Copy, Default)]
pub struct MoveHistory;
#[derive(Component)]
pub struct MoveHistoryEntry;
#[derive(Component, Clone, Copy, Default)]
pub struct MoveHistoryText;

/// UI scrolling event.
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Scroll {
    pub entity: Entity,
    /// Scroll delta in logical coordinates.
    pub delta: Vec2,
}

const LINE_HEIGHT: f32 = 24.0;

/// Injects scroll events into the UI hierarchy.
pub fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= LINE_HEIGHT;
        }

        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}
pub fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = &mut scroll.delta;
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.x > 0. {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.
        };

        if !max {
            scroll_position.x += delta.x;
            // Consume the X portion of the scroll delta.
            delta.x = 0.;
        }
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.y > 0. {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.
        };

        if !max {
            scroll_position.y += delta.y;
            // Consume the Y portion of the scroll delta.
            delta.y = 0.;
        }
    }

    // Stop propagating when the delta is fully consumed.
    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}

fn rebuild_move_history(
    query_display: Single<(Entity, Option<&Children>), With<MoveHistory>>,
    history: Res<ActiveGame>,
    mut commands: Commands,
) {
    let (entity, children) = *query_display;
    if let Some(children) = children {
        for &child in children {
            commands.entity(child).despawn();
        }
    }

    for (move_number, moves) in history.move_history.chunks(2).enumerate() {
        let white_move = &moves[0];
        let black_move = moves.get(1).map(String::as_str);

        commands.entity(entity).with_children(|parent| {
            spawn_move_history_row(parent, move_number + 1, white_move, black_move);
        });
    }
}

fn spawn_move_history_row(
    parent: &mut ChildSpawnerCommands,
    move_number: usize,
    white_move: &str,
    black_move: Option<&str>,
) {
    parent
        .spawn((
            MoveHistoryEntry,
            Node {
                width: percent(100),
                min_width: px(0),
                height: px(LINE_HEIGHT),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .with_children(|row| {
            spawn_move_history_cell(row, &format!("{move_number}."), 20.0);
            spawn_move_history_cell(row, white_move, 40.0);
            spawn_move_history_cell(row, black_move.unwrap_or_default(), 40.0);
        });
}

fn spawn_move_history_cell(parent: &mut ChildSpawnerCommands, text: &str, width: f32) {
    parent.spawn((
        MoveHistoryText,
        Node {
            width: percent(width),
            min_width: px(0),
            overflow: Overflow::clip(),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Text::new(text),
        TextLayout::no_wrap(),
    ));
}

pub fn refresh_move_history(
    _ev: On<MoveHistoryFullRefresh>,
    query_display: Single<(Entity, Option<&Children>), With<MoveHistory>>,
    history: Res<ActiveGame>,
    commands: Commands,
) {
    rebuild_move_history(query_display, history, commands);
}

pub fn update_move_history(
    _ev: On<MoveHistoryUpdated>,
    query_display: Single<(Entity, Option<&Children>), With<MoveHistory>>,
    history: Res<ActiveGame>,
    commands: Commands,
) {
    rebuild_move_history(query_display, history, commands);
}

pub fn resize_move_history_text(
    window: Single<&Window>,
    mut query: Query<&mut TextFont, With<MoveHistoryText>>,
) {
    let panel_width = window.width() * RIGHT_PANEL_RATIO;
    let font_size = FontSize::Px((panel_width * 0.06).clamp(8.0, 16.0));

    for mut text_font in &mut query {
        text_font.font_size = font_size;
    }
}
