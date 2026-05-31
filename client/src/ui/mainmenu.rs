use bevy::feathers::controls::{ButtonVariant, FeathersButton, FeathersScrollbar};
use bevy::feathers::display::{caption, label};
use bevy::feathers::theme::{InheritableThemeTextColor, ThemedText};
use bevy::feathers::tokens;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, ControlOrientation, ScrollArea};
use chess_core::{ClientId, GameId};
use chess_core::protocol::messages::ClientMessage;
use crate::client::lobby::LobbyState;
use crate::net::network::NetworkSend;
use crate::ui::theme::{default_button_size_180, wood_theme, THEME_BACKGROUND, THEME_BORDER, THEME_SURFACE, THEME_SURFACE_LIGHT, default_ui_shadow};
use crate::ui::{Overlay, Screen};

pub struct MainMenuPlugin;

#[derive(Component, Clone, Copy, Default)]
pub struct MainMenuRoot;

#[derive(Event)]
pub struct UpdateGamesList;

#[derive(Component, Clone, Copy, Default)]
pub struct GamesList;


impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Menu), (main_menu_scene.spawn(), request_games))
            .add_systems(OnExit(Screen::Menu), cleanup_main_menu)
            .add_observer(update_games_list)
            .insert_resource(wood_theme());
    }
}

/// Defining and creating the main menu scene.
fn main_menu_scene() -> impl Scene {
    bsn! {
        // root node
        Node {
            width: percent(100),
            height: percent(100),
            min_height: px(0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
        }
        MainMenuRoot
        BackgroundColor(THEME_BACKGROUND)
        InheritableThemeTextColor(tokens::TEXT_MAIN)
        Children [
            Node {
                height: px(220),
                padding: UiRect::all(px(28)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(22),
                border: UiRect::bottom(px(2)),
            }
            BackgroundColor(THEME_SURFACE)
            BorderColor::all(THEME_BORDER)
            Children [
                @label("Schach!")
                TextFont { font_size: FontSize::Rem(2.0) }
                TextLayout::justify(Justify::Center)
                --
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    column_gap: px(16),
                }
                Children [
                    @FeathersButton {
                        @caption: bsn! { @caption("New Game") },
                        @variant: ButtonVariant::Primary,
                    }
                    default_button_size_180()
                    on(|_: On<Activate>, mut next: ResMut<NextState<Overlay>>| {
                        next.set(Overlay::CreateGame);
                    })
                    --
                    @FeathersButton {
                        @caption: bsn! { @caption("Exit") },
                    }
                    default_button_size_180()
                    on(|_: On<Activate>, mut exit: MessageWriter<AppExit>| {
                        exit.write(AppExit::Success);
                    })
                ]
            ]
            --
            Node {
                flex_grow: 1.0,
                min_height: px(0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(px(28)),
                row_gap: px(14),
            }
            Children [
                @label("Open Games")
                TextFont { font_size: FontSize::Rem(1.5) }
                --
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
                BackgroundColor(THEME_SURFACE)
                BorderColor::all(THEME_BORDER)
                default_ui_shadow()
                Children [
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
                    Children [ @label("Loading games …") ]
                    --
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
                ]
            ]
        ]
    }
}

/// A reusable "mini scene" that represents a single row in the list of games.
/// A row is clickable and will spawn a JoinGame dialog for the attached game ID.
fn game_list_row_scene(label: String, game_id: GameId) -> impl Scene {
    bsn! {
        @FeathersButton {
            @caption: bsn! { @caption(label) }
        }
        Node {
            width: percent(100),
            min_height: px(50),
            justify_content: JustifyContent::Start,
            padding: UiRect::horizontal(px(14)),
            border: UiRect::bottom(px(1))
        }
        BorderColor::all(THEME_BORDER)
        BackgroundColor(THEME_SURFACE_LIGHT)
        on(move |_: On<Activate>, mut next: ResMut<NextState<Overlay>>, mut lobby: ResMut<LobbyState>| {
            lobby.pending_join_game = Some(game_id);
            next.set(Overlay::JoinGame);
        })
    }
}


/// Updates the list of games in the main menu.
/// This method is an observer and will be triggered on the UpdateGamesList event, which
/// is dispatched by the network thread when it receives changes of the list of games from the server.
fn update_games_list(
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

/// Requests the list of games from the server. This is a bevy sustem that is executed automatically
/// when the main menu scene is loaded.
fn request_games(mut commands: Commands) {
    commands.trigger(NetworkSend(
        ClientMessage::QueryGames,
    ));
}


/// Destructor of the main menu.
fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
