use bevy::feathers::controls::{ButtonVariant, FeathersButton};
use bevy::feathers::theme::{InheritableThemeTextColor, ThemedText};
use bevy::feathers::tokens;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use chess_core::protocol::messages::ClientMessage;
use chess_core::protocol::{NewGameParams, UserRoleSelection};

use crate::net::network::NetworkSend;
use crate::ui::theme::{WOOD_BORDER, WOOD_GOLD, WOOD_SURFACE};
use crate::ui::{Dialog, Overlay};

#[derive(Component, Clone, Copy, Default)]
struct TimeControlOption;

#[derive(Component, Clone, Copy, Default)]
struct ModeOption;

#[derive(Component, Clone, Copy, Default)]
struct SideOption;

#[derive(Resource)]
pub struct CreateGameSettings {
    pub mode: u8,
    pub time: u32,
    pub time_inc: u32,
    pub side: UserRoleSelection,
}

#[derive(Resource, Clone, Copy)]
pub struct PendingJoinSide(pub UserRoleSelection);

pub fn initialize_create_dialog(mut commands: Commands) {
    commands.insert_resource(CreateGameSettings {
        mode: 0,
        time: 600,
        time_inc: 10,
        side: UserRoleSelection::Random,
    });
}

pub fn create_dialog_scene() -> impl Scene {
    bsn! {
        Dialog
        Children[(
            Node {
                width: px(520), padding: UiRect::all(px(28)),
                flex_direction: FlexDirection::Column, row_gap: px(14),
                align_items: AlignItems::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(10)),
            }
            BackgroundColor(WOOD_SURFACE)
            BorderColor::all(WOOD_BORDER)
            BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.6), px(0), px(6), px(2), px(10))
            InheritableThemeTextColor(tokens::TEXT_MAIN)
            Children[
                (Text("New Game") ThemedText TextFont { font_size: FontSize::Px(28.0) } TextLayout::justify(Justify::Center)),
                (Node { width: percent(100), height: px(1) } BackgroundColor(WOOD_GOLD)),
                (Text("Time Control") ThemedText TextFont { font_size: FontSize::Px(18.0) } TextLayout::justify(Justify::Center)),
                (
                    Node { width: percent(100), flex_direction: FlexDirection::Row, justify_content: JustifyContent::Center, column_gap: px(10) }
                    Children[
                        (@FeathersButton { @caption: bsn! { Text("10 + 0") ThemedText } } TimeControlOption Node { width: px(135), height: px(46) } on(|activate: On<Activate>, mut settings: ResMut<CreateGameSettings>, mut commands: Commands, options: Query<Entity, With<TimeControlOption>>| {
                            select_time(&mut settings, 600, 0);
                            update_selected_option_button(&mut commands, activate.entity, options.iter());
                        })),
                        (@FeathersButton { @caption: bsn! { Text("10 + 10") ThemedText }, @variant: ButtonVariant::Primary } TimeControlOption Node { width: px(135), height: px(46) } on(|activate: On<Activate>, mut settings: ResMut<CreateGameSettings>, mut commands: Commands, options: Query<Entity, With<TimeControlOption>>| {
                            select_time(&mut settings, 600, 10);
                            update_selected_option_button(&mut commands, activate.entity, options.iter());
                        })),
                        (@FeathersButton { @caption: bsn! { Text("5 + 3") ThemedText } } TimeControlOption Node { width: px(135), height: px(46) } on(|activate: On<Activate>, mut settings: ResMut<CreateGameSettings>, mut commands: Commands, options: Query<Entity, With<TimeControlOption>>| {
                            select_time(&mut settings, 300, 3);
                            update_selected_option_button(&mut commands, activate.entity, options.iter());
                        }))
                    ]
                ),
                (Text("Join As") ThemedText TextFont { font_size: FontSize::Px(18.0) } TextLayout::justify(Justify::Center)),
                (
                    Node { width: percent(100), flex_direction: FlexDirection::Row, justify_content: JustifyContent::Center, column_gap: px(10) }
                    Children[
                        (@FeathersButton { @caption: bsn! { Text("White") ThemedText } } SideOption Node { width: px(135), height: px(46) } on(|activate: On<Activate>, mut settings: ResMut<CreateGameSettings>, mut commands: Commands, options: Query<Entity, With<SideOption>>| {
                            select_side(&mut settings, UserRoleSelection::White);
                            update_selected_option_button(&mut commands, activate.entity, options.iter());
                        })),
                        (@FeathersButton { @caption: bsn! { Text("Black") ThemedText } } SideOption Node { width: px(135), height: px(46) } on(|activate: On<Activate>, mut settings: ResMut<CreateGameSettings>, mut commands: Commands, options: Query<Entity, With<SideOption>>| {
                            select_side(&mut settings, UserRoleSelection::Black);
                            update_selected_option_button(&mut commands, activate.entity, options.iter());
                        })),
                        (@FeathersButton { @caption: bsn! { Text("Random") ThemedText }, @variant: ButtonVariant::Primary } SideOption Node { width: px(135), height: px(46) } on(|activate: On<Activate>, mut settings: ResMut<CreateGameSettings>, mut commands: Commands, options: Query<Entity, With<SideOption>>| {
                            select_side(&mut settings, UserRoleSelection::Random);
                            update_selected_option_button(&mut commands, activate.entity, options.iter());
                        }))
                    ]
                ),
                (Node { width: percent(100), height: px(1) } BackgroundColor(WOOD_BORDER)),
                (
                    Node { width: percent(100), flex_direction: FlexDirection::Row, justify_content: JustifyContent::Center, column_gap: px(14), padding: UiRect::top(px(4)) }
                    Children[
                        (@FeathersButton { @caption: bsn! { Text("Cancel") ThemedText } }
                            Node { width: px(150), height: px(50) }
                            on(|_: On<Activate>, mut next: ResMut<NextState<Overlay>>| {
                                next.set(Overlay::None);
                            })),
                        (@FeathersButton { @caption: bsn! { Text("Create Game") ThemedText }, @variant: ButtonVariant::Primary }
                            Node { width: px(180), height: px(50) }
                            on(confirm_create_game))
                    ]
                )
            ]
        )]
    }
}

pub fn cleanup_create_dialog(mut commands: Commands, query: Query<Entity, With<Dialog>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<CreateGameSettings>();
}

fn select_time(settings: &mut CreateGameSettings, time: u32, time_inc: u32) {
    settings.time = time;
    settings.time_inc = time_inc;
}

fn select_side(settings: &mut CreateGameSettings, side: UserRoleSelection) {
    settings.side = side;
}

fn update_selected_option_button(
    commands: &mut Commands,
    selected: Entity,
    options: impl Iterator<Item = Entity>,
) {
    for entity in options {
        commands.entity(entity).insert(if entity == selected {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Normal
        });
    }
}

fn confirm_create_game(
    _: On<Activate>,
    settings: Res<CreateGameSettings>,
    mut commands: Commands,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    commands.trigger(NetworkSend(ClientMessage::NewGame(NewGameParams {
        mode: settings.mode,
        time: settings.time,
        time_inc: settings.time_inc,
    })));
    commands.insert_resource(PendingJoinSide(settings.side));
    next_overlay.set(Overlay::None);
}
