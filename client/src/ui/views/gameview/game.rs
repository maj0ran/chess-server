use crate::state::{ClientState, Overlay, Screen};
use crate::ui::views::gameview::dialogs::game_over_dialog::{
    cleanup_game_over_dialog, game_over_dialog_action_system, setup_game_over_dialog,
};
use crate::ui::views::gameview::dialogs::quit_game_dialog::{
    cleanup_quit_game_dialog, quit_game_dialog_action_system, setup_quit_game_dialog,
};
use crate::ui::{COLOR_LIGHT2, COLOR_MID};
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use bevy_flair::prelude::*;
use chess_core::{ChessMove, ClientMessage, SpecialMove, Tile};
use std::collections::HashMap;

#[derive(Component)]
pub struct GameScreenComponent;

#[derive(Component)]
pub struct Board; // marker for board square entities

#[derive(Component)]
pub struct PieceEntity {
    pub tile: String,
    pub piece: char,
}

#[derive(Resource)]
pub struct GameAssets {
    pub piece_textures: HashMap<char, Handle<Image>>,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Game view: draw board, drag pieces, update game state
        app.add_systems(OnEnter(Screen::Game), setup_game)
            .add_systems(OnExit(Screen::Game), cleanup_game)
            .add_systems(Update, update_game.run_if(in_state(Screen::Game)))
            .add_systems(Update, handle_drag.run_if(in_state(Screen::Game)))
            .add_systems(Update, on_resize_board.run_if(in_state(Screen::Game)))
            // listening for keyboard input (only ESC for now)
            .add_systems(Update, listen_keyboard_input.run_if(in_state(Screen::Game)))
            // The promotion dialog that appears when a pawn is dragged on 8th rank
            .add_systems(OnEnter(Overlay::Promotion), promotion_dialog)
            .add_systems(OnExit(Overlay::Promotion), cleanup_promotion_dialog)
            .add_systems(
                Update,
                promotion_action_system.run_if(in_state(Overlay::Promotion)),
            )
            // Game over dialog on checkmate/stalemate/resign
            .add_systems(OnEnter(Overlay::GameOver), setup_game_over_dialog)
            .add_systems(OnExit(Overlay::GameOver), cleanup_game_over_dialog)
            .add_systems(
                Update,
                game_over_dialog_action_system.run_if(in_state(Overlay::GameOver)),
            )
            // Quit game dialog when pressing Escape
            .add_systems(OnEnter(Overlay::QuitGameDialog), setup_quit_game_dialog)
            .add_systems(OnExit(Overlay::QuitGameDialog), cleanup_quit_game_dialog)
            .add_systems(
                Update,
                quit_game_dialog_action_system.run_if(in_state(Overlay::QuitGameDialog)),
            );
    }
}

pub fn listen_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_overlay.set(Overlay::QuitGameDialog);
    }
}

pub fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    win_res: Res<crate::WindowSize>,
    q_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let mut piece_textures = HashMap::new();
    let pieces = [
        ('r', "r_b.png"),
        ('n', "n_b.png"),
        ('b', "b_b.png"),
        ('q', "q_b.png"),
        ('k', "k_b.png"),
        ('p', "p_b.png"),
        ('R', "r_w.png"),
        ('N', "n_w.png"),
        ('B', "b_w.png"),
        ('Q', "q_w.png"),
        ('K', "k_w.png"),
        ('P', "p_w.png"),
    ];
    for (key, path) in pieces {
        piece_textures.insert(key, asset_server.load(path.to_string()));
    }
    commands.insert_resource(GameAssets { piece_textures });

    commands
        .spawn((Transform::default(), GameScreenComponent))
        .with_children(|parent| {
            // Spawn Board squares with dynamic sizing
            let square_size = if win_res.width > 0.0 && win_res.height > 0.0 {
                0.8 * win_res.width.min(win_res.height) / 8.0
            } else if let Ok(window) = q_window.single() {
                0.8 * window.width().min(window.height()) / 8.0
            } else {
                60.0
            };
            draw_board(parent, square_size);
        });
}

pub fn draw_board(parent: &mut RelatedSpawnerCommands<ChildOf>, square_size: f32) {
    for r in 0..8 {
        for f in 0..8 {
            let color = if (r + f) % 2 == 0 {
                COLOR_LIGHT2
            } else {
                COLOR_MID
            };
            parent.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(square_size, square_size)),
                    ..default()
                },
                Transform::from_xyz(
                    f as f32 * square_size - 3.5 * square_size,
                    (7 - r) as f32 * square_size - 3.5 * square_size,
                    0.0,
                ),
                Board,
            ));
        }
    }
}

pub fn cleanup_game(mut commands: Commands, query: Query<Entity, With<GameScreenComponent>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
pub struct Draggable;

#[derive(Component)]
pub struct Dragging;

pub fn update_game(
    mut state: ResMut<ClientState>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    win_res: Res<crate::WindowSize>,
    q_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    pieces_query: Query<(Entity, &PieceEntity), Without<Dragging>>,
    root_query: Query<Entity, With<GameScreenComponent>>,
) {
    let root = match root_query.single() {
        Ok(r) => r,
        Err(_) => return,
    };

    // Only update the board if the state has changed (marked as dirty by the network system)
    if !state.game_state.dirty {
        return;
    }
    state.game_state.dirty = false;

    // Remove all existing piece entities to redraw the board from the new state.
    // We skip pieces currently being dragged to avoid interrupting the user's action.
    for (entity, _) in pieces_query.iter() {
        commands.entity(entity).despawn();
    }

    let board_len = state.game_state.board.len();
    if board_len == 0 {
        return;
    }

    // Determine current square size
    let square_size = if win_res.width > 0.0 && win_res.height > 0.0 {
        0.8 * win_res.width.min(win_res.height) / 8.0
    } else if let Ok(window) = q_window.single() {
        0.8 * window.width().min(window.height()) / 8.0
    } else {
        60.0
    };

    // Spawn new piece entities for each piece in the current board state
    commands.entity(root).with_children(|parent| {
        for (tile_name, &piece) in &state.game_state.board {
            // Convert algebraic notation (e.g., "e2") to 0-7 indices
            // f: file (a-h), r: rank (1-8)
            let f = (tile_name.as_bytes()[0] as char).to_ascii_lowercase() as u8 - b'a';
            let r = tile_name.as_bytes()[1] - b'1';

            if let Some(texture) = assets.piece_textures.get(&piece) {
                parent.spawn((
                    Sprite {
                        image: texture.clone(),
                        custom_size: Some(Vec2::new(square_size, square_size)),
                        ..default()
                    },
                    // Position the piece in the center of its square.
                    Transform::from_xyz(
                        f as f32 * square_size - 3.5 * square_size,
                        r as f32 * square_size - 3.5 * square_size,
                        1.0,
                    ),
                    PieceEntity {
                        tile: tile_name.clone(),
                        piece,
                    },
                    Draggable,
                ));
            }
        }
    });
}

pub fn handle_drag(
    mut commands: Commands,
    mut state: ResMut<ClientState>,
    buttons: Res<ButtonInput<MouseButton>>,
    win_res: Res<crate::WindowSize>,
    q_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_draggable: Query<
        (Entity, &mut Transform, &PieceEntity),
        (With<Draggable>, Without<Dragging>),
    >,
    mut q_dragging: Query<(Entity, &mut Transform, &PieceEntity), With<Dragging>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    let Ok(window) = q_window.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };

    let mouse_pos = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok());

    // Current square size for interaction calculations
    let square_size = if win_res.width > 0.0 && win_res.height > 0.0 {
        0.8 * win_res.width.min(win_res.height) / 8.0
    } else if let Ok(window) = q_window.single() {
        0.8 * window.width().min(window.height()) / 8.0
    } else {
        60.0
    };

    if let Some(world_pos) = mouse_pos {
        if buttons.just_pressed(MouseButton::Left) {
            // Check if any piece is being clicked on
            for (entity, transform, _piece) in q_draggable.iter() {
                let dist = (transform.translation.truncate() - world_pos).length();
                // Use half square size as hit radius
                if dist < square_size * 0.5 {
                    if let Ok(mut entity_commands) = commands.get_entity(entity) {
                        // Mark the entity as being dragged
                        entity_commands.insert(Dragging);
                    }
                    // Only drag one piece at a time
                    break;
                }
            }
        }

        if buttons.pressed(MouseButton::Left) {
            // Update the position of all pieces currently being dragged to follow the mouse
            for (_, mut transform, _) in q_dragging.iter_mut() {
                transform.translation.x = world_pos.x;
                transform.translation.y = world_pos.y;
                transform.scale.x = 1.3;
                transform.scale.y = 1.3;
            }
        }

        if buttons.just_released(MouseButton::Left) {
            for (entity, mut transform, piece) in q_dragging.iter_mut() {
                // Calculate target square from world coordinates using dynamic square size
                let f_dst = ((transform.translation.x + 3.5 * square_size + square_size * 0.5)
                    / square_size)
                    .floor() as i32;
                let r_dst = ((transform.translation.y + 3.5 * square_size + square_size * 0.5)
                    / square_size)
                    .floor() as i32;

                // Check if the release point is within the board bounds
                if f_dst >= 0 && f_dst < 8 && r_dst >= 0 && r_dst < 8 {
                    let dst_tile_str = format!(
                        "{}{}",
                        (97 + f_dst) as u8 as char, // ASCII 'a' = 97
                        (r_dst + 49) as u8 as char  // ASCII '1' = 49
                    );

                    let src = Tile::from(&*piece.tile);
                    let dst = Tile::from(&*dst_tile_str);

                    // Pawn promotion check
                    let is_pawn = piece.piece.to_ascii_lowercase() == 'p';
                    let is_promotion_rank = (dst.rank == '8' && piece.piece == 'P')
                        || (dst.rank == '1' && piece.piece == 'p');

                    if is_pawn && is_promotion_rank {
                        state.game_state.pending_promotion = Some((src, dst));
                        next_overlay.set(Overlay::Promotion);
                    } else if let Some(game_id) = state.menu_state.selected_game {
                        state.network.send(ClientMessage::Move(
                            game_id,
                            ChessMove {
                                src,
                                dst,
                                special: None,
                            },
                        ));
                    }
                }

                // Finalize the drag by removing the Dragging component.
                if let Ok(mut e) = commands.get_entity(entity) {
                    e.remove::<Dragging>();
                }
                transform.scale.x = 1.0;
                transform.scale.y = 1.0;
            }
        }
    }
}
#[derive(Component)]
pub enum PromotionAction {
    Queen,
    Knight,
    Rook,
    Bishop,
    Cancel,
}

#[derive(Component)]
pub struct PromotionDialogComponent;

// Redraw board and resize pieces on window resize only
pub fn on_resize_board(
    mut resize_reader: MessageReader<bevy::window::WindowResized>,
    mut commands: Commands,
    root_query: Query<Entity, With<GameScreenComponent>>,
    board_query: Query<Entity, With<Board>>,
    mut pieces_query: Query<(&mut Sprite, &mut Transform, &PieceEntity), Without<Dragging>>,
) {
    // Trigger only when there was at least one resize event
    let mut new_size = None;
    for e in resize_reader.read() {
        new_size = Some(Vec2::new(e.width, e.height));
    }
    let Some(size) = new_size else {
        return;
    };

    let Ok(root) = root_query.single() else {
        return;
    };

    let square_size = 0.8 * size.x.min(size.y) / 8.0;

    // Remove old board squares
    for e in board_query.iter() {
        commands.entity(e).despawn();
    }

    // Recreate board squares
    commands.entity(root).with_children(|parent| {
        draw_board(parent, square_size);
    });

    // Resize and reposition pieces (skip currently dragged ones)
    for (mut sprite, mut transform, piece) in pieces_query.iter_mut() {
        sprite.custom_size = Some(Vec2::new(square_size, square_size));
        let f = (piece.tile.as_bytes()[0].to_ascii_lowercase() - b'a') as f32;
        let r = (piece.tile.as_bytes()[1] - b'1') as f32;
        transform.translation.x = f * square_size - 3.5 * square_size;
        transform.translation.y = r * square_size - 3.5 * square_size;
    }
}

pub fn promotion_dialog(
    mut commands: Commands,
    assets: Res<GameAssets>,
    assets_server: Res<AssetServer>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            NodeStyleSheet::new(assets_server.load("style.css")),
            PromotionDialogComponent,
            ClassList::new("dialog-overlay"),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ClassList::new("dialog-content"),
                ))
                .with_children(|p| {
                    p.spawn((Node::default(), ClassList::new("promotion-row")))
                        .with_children(|row| {
                            let pieces = [
                                ('Q', PromotionAction::Queen),
                                ('R', PromotionAction::Rook),
                                ('B', PromotionAction::Bishop),
                                ('N', PromotionAction::Knight),
                            ];

                            for (c, action) in pieces {
                                row.spawn((
                                    Button,
                                    Interaction::default(),
                                    ClassList::new("promotion-button"),
                                    action,
                                ))
                                .with_children(|btn| {
                                    if let Some(texture) = assets.piece_textures.get(&c) {
                                        btn.spawn((
                                            ImageNode {
                                                image: texture.clone(),
                                                ..default()
                                            },
                                            ClassList::new("piece-icon"),
                                        ));
                                    }
                                });
                            }
                            row.spawn((
                                Button,
                                Interaction::default(),
                                ClassList::new("button-red"),
                                PromotionAction::Cancel,
                            ))
                            .with_children(|btn| {
                                btn.spawn(Text::new("X"));
                            });
                        });
                });
        });
}

pub fn cleanup_promotion_dialog(
    mut commands: Commands,
    query: Query<Entity, With<PromotionDialogComponent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn promotion_action_system(
    mut interaction_query: Query<
        (&Interaction, &PromotionAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<ClientState>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            if let Some((src, dst)) = state.game_state.pending_promotion.take() {
                let special = match action {
                    PromotionAction::Queen => Some(SpecialMove::Queen),
                    PromotionAction::Rook => Some(SpecialMove::Rook),
                    PromotionAction::Bishop => Some(SpecialMove::Bishop),
                    PromotionAction::Knight => Some(SpecialMove::Knight),
                    PromotionAction::Cancel => None,
                };

                if let Some(game_id) = state.menu_state.selected_game {
                    state.network.send(ClientMessage::Move(
                        game_id,
                        ChessMove { src, dst, special },
                    ));
                }
            }
            next_overlay.set(Overlay::None);
        }
    }
}
