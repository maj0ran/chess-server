use crate::state::{ClientState, Overlay, Screen};
use crate::ui::views::gameview::game_over_dialog::{
    cleanup_game_over_dialog, game_over_dialog_action_system, setup_game_over_dialog,
};
use crate::ui::views::gameview::quit_game_dialog::{
    cleanup_quit_game_dialog, quit_game_dialog_action_system, setup_quit_game_dialog,
};
use crate::ui::*;
use crate::{spawn_button, spawn_dialog, spawn_label};
use bevy::prelude::*;
use chess_core::{ChessMove, ClientMessage, SpecialMove, Tile};
use std::collections::HashMap;

#[derive(Component)]
pub struct GameScreenComponent;

#[derive(Component)]
pub struct Board;

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

pub fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
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
        .spawn((
            Transform::default(),
            Visibility::default(),
            GameScreenComponent,
        ))
        .with_children(|parent| {
            // Spawn Board squares
            for r in 0..8 {
                for f in 0..8 {
                    let color = if (r + f) % 2 == 0 {
                        COLOR_LIGHT
                    } else {
                        COLOR_MID
                    };
                    parent.spawn((
                        Sprite {
                            color,
                            custom_size: Some(Vec2::new(60.0, 60.0)),
                            ..default()
                        },
                        Transform::from_xyz(
                            f as f32 * 60.0 - 3.5 * 60.0,
                            (7 - r) as f32 * 60.0 - 3.5 * 60.0,
                            0.0,
                        ),
                    ));
                }
            }
        });
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
                        custom_size: Some(Vec2::new(60.0, 60.0)),
                        ..default()
                    },
                    // Position the piece in the center of its square.
                    // The board is centered at (0,0), squares are 60x60 units.
                    Transform::from_xyz(
                        f as f32 * 60.0 - 3.5 * 60.0,
                        r as f32 * 60.0 - 3.5 * 60.0,
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

    if let Some(world_pos) = mouse_pos {
        if buttons.just_pressed(MouseButton::Left) {
            // Check if any piece is being clicked on
            for (entity, transform, _piece) in q_draggable.iter() {
                let dist = (transform.translation.truncate() - world_pos).length();
                // Pieces are 60x60, so 30.0 radius is a good hit area
                if dist < 30.0 {
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
                // Calculate target square from world coordinates
                // The board is 8x8 squares of 60.0 units each, centered around (0,0)
                // -3.5 * 60.0 is the center of the first column/row
                let f_dst = ((transform.translation.x + 3.5 * 60.0 + 30.0) / 60.0).floor() as i32;
                let r_dst = ((transform.translation.y + 3.5 * 60.0 + 30.0) / 60.0).floor() as i32;

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

pub fn promotion_dialog(mut commands: Commands, assets: Res<GameAssets>) {
    spawn_dialog!(
        commands,
        PromotionDialogComponent,
        Val::Px(350.0),
        Val::Px(120.0),
        |p| {
            p.spawn((Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                ..default()
            },))
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
                            Node {
                                width: Val::Px(60.0),
                                height: Val::Px(60.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ButtonColors::default(),
                            action,
                        ))
                        .with_children(|btn| {
                            if let Some(texture) = assets.piece_textures.get(&c) {
                                btn.spawn((
                                    ImageNode {
                                        image: texture.clone(),
                                        ..default()
                                    },
                                    Node {
                                        width: Val::Px(50.0),
                                        height: Val::Px(50.0),
                                        ..default()
                                    },
                                ));
                            }
                        });
                    }
                    spawn_button!(row, "X", PromotionAction::Cancel, ButtonColors::red());
                });
        }
    );
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
