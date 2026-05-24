use bevy::feathers::controls::{ButtonVariant, FeathersButton, FeathersScrollbar};
use bevy::feathers::theme::{InheritableThemeTextColor, ThemedText};
use bevy::feathers::tokens;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, ControlOrientation, ScrollArea};
use chess_core::{ClientId, GameId};

use crate::client::lobby::LobbyState;
use crate::net::network::NetworkSend;
use crate::ui::theme::{
    WOOD_BACKGROUND, WOOD_BORDER, WOOD_SURFACE, WOOD_SURFACE_LIGHT, wood_theme,
};
use crate::ui::{Overlay, Screen};

pub struct MainMenuPlugin;

#[derive(Component, Clone, Copy, Default)]
pub struct MainMenuRoot;

#[derive(Event)]
pub struct UpdateGamesList;

#[derive(Component, Clone, Copy, Default)]
pub struct GamesList;

fn game_list_row_scene(label: String, game_id: GameId) -> impl Scene {
    bsn! {
        @FeathersButton {
            @caption: bsn! { Text(label) ThemedText }
        }
        Node { width: percent(100), min_height: px(46), justify_content: JustifyContent::Start, padding: UiRect::horizontal(px(14)), border: UiRect::bottom(px(1)) }
        BorderColor::all(WOOD_BORDER)
        BackgroundColor(WOOD_SURFACE_LIGHT)
        on(move |_: On<Activate>, mut next: ResMut<NextState<Overlay>>, mut lobby: ResMut<LobbyState>| {
            lobby.pending_join_game = Some(game_id);
            next.set(Overlay::JoinGame);
        })
    }
}

pub fn update_games_list(
    _event: On<UpdateGamesList>,
    lobby: Res<LobbyState>,
    mut commands: Commands,
    container: Single<(Entity, Option<&Children>), With<GamesList>>,
) {
    let (container, children) = *container;
    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    if lobby.get_games().is_empty() {
        commands.entity(container).with_children(|parent| {
            parent.spawn((Text::new("No open games"), ThemedText));
        });
        return;
    }

    for (gid, details) in lobby.get_games() {
        let white = details
            .white_player
            .and_then(|id| lobby.get_client_info(id))
            .cloned()
            .unwrap_or_else(|| {
                details
                    .white_player
                    .map(|id: ClientId| id.to_string())
                    .unwrap_or_else(|| "–".into())
            });
        let black = details
            .black_player
            .and_then(|id| lobby.get_client_info(id))
            .cloned()
            .unwrap_or_else(|| {
                details
                    .black_player
                    .map(|id: ClientId| id.to_string())
                    .unwrap_or_else(|| "–".into())
            });
        let label = format!("{white} vs {black}");
        let game_id = *gid;
        let mut row = commands.spawn_scene(game_list_row_scene(label, game_id));
        row.insert(ChildOf(container));
    }
}

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Menu), (scene.spawn(), request_games))
            .add_systems(OnExit(Screen::Menu), cleanup_main_menu)
            .add_observer(update_games_list)
            .insert_resource(wood_theme());
    }
}

fn scene() -> impl SceneList {
    bsn_list![main_menu()]
}

fn main_menu() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            min_height: px(0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
        }
        MainMenuRoot
        BackgroundColor(WOOD_BACKGROUND)
        InheritableThemeTextColor(tokens::TEXT_MAIN)
        Children[
            (
                Node {
                    height: px(220),
                    padding: UiRect::all(px(28)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(22),
                    border: UiRect::bottom(px(2)),
                }
                BackgroundColor(WOOD_SURFACE)
                BorderColor::all(WOOD_BORDER)
                Children[
                    (Text("Schach!") ThemedText TextFont { font_size: FontSize::Px(38.0) } TextLayout::justify(Justify::Center)),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            column_gap: px(16),
                        }
                        Children[
                            (
                                @FeathersButton {
                                    @caption: bsn! { Text("New Game") ThemedText },
                                    @variant: ButtonVariant::Primary,
                                }
                                Node { width: px(190), height: px(56) }
                                on(|_: On<Activate>, mut next: ResMut<NextState<Overlay>>| {
                                    next.set(Overlay::CreateGame);
                                })
                            ),
                            (
                                @FeathersButton {
                                    @caption: bsn! { Text("Exit") ThemedText },
                                }
                                on(|_: On<Activate>, mut exit: MessageWriter<AppExit>| {
                                    exit.write(AppExit::Success);
                                })
                                Node { width: px(150), height: px(56) }
                            )
                        ]
                    )
                ]
            ),
            (
                Node {
                    flex_grow: 1.0,
                    min_height: px(0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    padding: UiRect::all(px(28)),
                    row_gap: px(14),
                }
                Children[
                    (Text("Open Games") ThemedText TextFont { font_size: FontSize::Px(24.0) }),
                    (
                        Node {
                            flex_grow: 1.0,
                            min_height: px(0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            padding: UiRect::all(px(14)),
                            border: UiRect::all(px(2)),
                            border_radius: BorderRadius::all(px(8)),
                            overflow: Overflow::clip(),
                        }
                        BackgroundColor(WOOD_SURFACE)
                        BorderColor::all(WOOD_BORDER)
                        BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.45), px(0), px(4), px(1), px(7))
                        Children[
                            (
                                #games_list_scroll_area
                                GamesList
                                Node {
                                    flex_grow: 1.0,
                                    min_height: px(0),
                                    width: percent(100),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Stretch,
                                    overflow: Overflow::scroll_y(),
                                    padding: UiRect::right(px(10)),
                                }
                                ScrollArea
                                Children[(Text("Loading games …") ThemedText)]
                            ),
                            (
                                @FeathersScrollbar {
                                    @target: #games_list_scroll_area,
                                    @orientation: {ControlOrientation::Vertical},
                                }
                                Node {
                                    position_type: PositionType::Absolute,
                                    right: px(4),
                                    top: px(4),
                                    bottom: px(4),
                                    width: px(6),
                                }
                            )
                        ]
                    )
                ]
            )
        ]
    }
}

fn request_games(mut commands: Commands) {
    commands.trigger(NetworkSend(
        chess_core::protocol::messages::ClientMessage::QueryGames,
    ));
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
