use crate::client::game::ActiveGame;
use crate::ui::views::gameview::GameScreenComponent;
use bevy::color::Color;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;

#[derive(Event)]
pub struct MoveHistoryUpdated;
#[derive(Event)]
pub struct MoveHistoryFullRefresh;

#[derive(Component)]
pub struct MoveHistory;
#[derive(Component)]
pub struct MoveHistoryEntry;

/// UI scrolling event.
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Scroll {
    pub entity: Entity,
    /// Scroll delta in logical coordinates.
    pub delta: Vec2,
}

const LINE_HEIGHT: f32 = 24.0;
const MOVE_HISTORY_FONT_SIZE: f32 = 18.0;

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

impl MoveHistory {
    pub fn new() -> (GameScreenComponent, MoveHistory, Node) {
        (
            GameScreenComponent,
            MoveHistory,
            Node {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                height: Val::Percent(100.0),
                width: Val::Px(400.0),
                top: Val::Px(100.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
        )
    }
}

pub fn refresh_move_history(
    _ev: On<MoveHistoryFullRefresh>,
    query_display: Single<(Entity, Option<&Children>), With<MoveHistory>>,
    history: ResMut<ActiveGame>,
    mut commands: Commands,
) {
    let (entity, children) = *query_display;
    if let Some(children) = children {
        for &child in children {
            commands.entity(child).despawn();
        }
    }

    let history = &history.move_history;

    let mut current_full_move_text = String::new();
    for (i, move_str) in history.iter().enumerate() {
        let half_move_num = i + 1;
        if half_move_num % 2 == 1 {
            let move_num = half_move_num / 2 + 1;
            let move_num_spacing = " ".repeat(4 - move_num.to_string().len());
            let half_move_spacing = " ".repeat(10 - move_str.len());
            current_full_move_text = format!(
                "{}.{}{}{}",
                move_num, move_num_spacing, move_str, &half_move_spacing
            );

            // If it's the last move, we should spawn it now.
            if i == history.len() - 1 {
                let text_to_spawn = current_full_move_text.clone();
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        MoveHistoryEntry,
                        Text::new(text_to_spawn),
                        TextFont {
                            font_size: MOVE_HISTORY_FONT_SIZE,
                            ..default()
                        },
                    ));
                });
            }
        } else {
            current_full_move_text.push_str(move_str);
            let text_to_spawn = current_full_move_text.clone();
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    MoveHistoryEntry,
                    Text::new(text_to_spawn),
                    TextFont {
                        font_size: MOVE_HISTORY_FONT_SIZE,
                        ..default()
                    },
                ));
            });
            current_full_move_text.clear();
        }
    }
}

pub fn update_move_history(
    _ev: On<MoveHistoryUpdated>,
    query_display: Single<(Entity, Option<&Children>), With<MoveHistory>>,
    history: ResMut<ActiveGame>,
    mut query_text: Query<&mut Text>,
    mut commands: Commands,
) {
    let history = &history.move_history;
    let half_move_num = history.len();
    if half_move_num == 0 {
        return;
    }

    let (entity, children) = *query_display;

    let last_move = history.last().unwrap();
    if half_move_num % 2 == 1 {
        let move_num = half_move_num / 2 + 1;
        let move_num_spacing = " ".repeat(4 - move_num.to_string().len());
        let half_move_spacing = " ".repeat(10 - last_move.len());
        let text = format!(
            "{}.{}{}{}",
            move_num, move_num_spacing, last_move, &half_move_spacing
        );
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                MoveHistoryEntry,
                Text::new(text),
                TextFont {
                    font_size: MOVE_HISTORY_FONT_SIZE,
                    ..default()
                },
            ));
        });
    } else {
        if let Some(children) = children {
            if let Some(&last_child) = children.last() {
                if let Ok(mut text) = query_text.get_mut(last_child) {
                    text.0.push_str(last_move);
                }
            }
        }
    }
}
