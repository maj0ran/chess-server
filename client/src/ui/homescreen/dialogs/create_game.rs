use bevy::{
    prelude::*,
    feathers::tokens,
    feathers::theme::InheritableThemeTextColor,
    feathers::rounded_corners::RoundedCorners,
    feathers::display::{caption, label},
    feathers::controls::{ButtonVariant, FeathersButton},
    ui_widgets::Activate
};

use crate::{
    backend::net::NetworkSend,
    ui::theme,
    ui::{Dialog, Overlay},
};

use chess_core::protocol::messages::ClientMessage;
use chess_core::protocol::{NewGameParams, UserRoleSelection};


#[derive(Component, Clone, Copy, Default)]
struct TimeControlOptionGroup;
#[derive(Component, Clone, Copy, Default)]
struct SideOptionGroup;

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
        Children [
            theme::default_dialog_node()
            theme::default_ui_shadow()
            BackgroundColor(theme::THEME_SURFACE)
            BorderColor::all(theme::THEME_BORDER)
            InheritableThemeTextColor(tokens::TEXT_MAIN)
            Children [
                @label("New Game")
                TextFont { font_size: FontSize::Rem(1.5) }
                TextLayout::justify(Justify::Center)
                --
                // line separator
                Node { width: percent(100), height: px(1) }
                BackgroundColor(theme::THEME_GOLD)
                --
                @label("Time Control")
                TextFont { font_size: FontSize::Rem(1.0) }
                --
                theme::default_button_row()
                Children [
                    @FeathersButton { @caption: bsn! { @caption("10 + 0") } }
                    theme::default_button_size_120()
                    TimeControlOptionGroup
                    on(|activate: On<Activate>, mut settings: ResMut<CreateGameSettings>, mut commands: Commands, options: Query<Entity, With<TimeControlOptionGroup >>| {
                        select_time(&mut settings, 600, 0);
                        update_selected_option_button(&mut commands, activate.entity, options.iter());
                    })
                    --
                    @FeathersButton { @caption: bsn! { @caption("10 + 10") }, @variant: ButtonVariant::Primary }
                    theme::default_button_size_120()
                    TimeControlOptionGroup
                    on(|activate: On<Activate>, mut settings: ResMut<CreateGameSettings>, mut commands: Commands, options: Query<Entity, With<TimeControlOptionGroup >>| {
                        select_time(&mut settings, 600, 10);
                        update_selected_option_button(&mut commands, activate.entity, options.iter());
                    })
                    --
                    @FeathersButton { @caption: bsn! { @caption("5 + 3") } }
                    theme::default_button_size_120()
                    TimeControlOptionGroup
                    on(|activate: On<Activate>, mut settings: ResMut<CreateGameSettings>, mut commands: Commands, options: Query<Entity, With<TimeControlOptionGroup >>| {
                        select_time(&mut settings, 300, 3);
                        update_selected_option_button(&mut commands, activate.entity, options.iter());
                    })
                ]
                --
                @label("Join As")
                TextFont { font_size: FontSize::Rem(1.0) }
                --
                theme::default_button_row()
                Children [
                    @FeathersButton { @caption: bsn! { @caption("White") }, @corners: RoundedCorners::All }
                    theme::default_button_size_120()
                    SideOptionGroup
                    on(|activate: On<Activate>,
                        mut settings: ResMut<CreateGameSettings>,
                        mut commands: Commands,
                        button_group: Query<Entity, With<SideOptionGroup >>| {
                            select_side(&mut settings, UserRoleSelection::White);
                            update_selected_option_button(&mut commands, activate.entity, button_group.iter());
                    })
                    --
                    @FeathersButton { @caption: bsn! { @caption("Black") } }
                    theme::default_button_size_120()
                    SideOptionGroup
                    on(|activate: On<Activate>,
                        mut settings: ResMut<CreateGameSettings>,
                        mut commands: Commands,
                        button_group: Query<Entity, With<SideOptionGroup >>| {
                            select_side(&mut settings, UserRoleSelection::Black);
                            update_selected_option_button(&mut commands, activate.entity, button_group.iter());
                    })
                    --
                    @FeathersButton { @caption: bsn! { @caption("Random") }, @variant: ButtonVariant::Primary, @corners: RoundedCorners::All }
                    theme::default_button_size_120()
                    SideOptionGroup
                        on(|activate: On<Activate>,
                        mut settings: ResMut<CreateGameSettings>,
                        mut commands: Commands,
                        button_group: Query<Entity, With<SideOptionGroup >>| {
                            select_side(&mut settings, UserRoleSelection::Random);
                            update_selected_option_button(&mut commands, activate.entity, button_group.iter());
                    })
                ]
                --
                Node { width: percent(100), height: px(1) }
                BackgroundColor(theme::THEME_BORDER)
                --
                theme::default_button_row()
                Children [
                    @FeathersButton { @caption: bsn! { @caption("Cancel") } }
                        theme::default_button_size_180()
                        on(|_: On<Activate>, mut next: ResMut<NextState<Overlay>>| {
                            next.set(Overlay::None);
                        })
                    --
                    @FeathersButton { @caption: bsn! { @caption("Create Game") }, @variant: ButtonVariant::Primary }
                        theme::default_button_size_180()
                        on(confirm_create_game)
                ]
            ]
        ]
    }
}

/// Callback for the time selection buttons
fn select_time(settings: &mut CreateGameSettings, time: u32, time_inc: u32) {
    settings.time = time;
    settings.time_inc = time_inc;
}

/// Callback for the side selection buttons
fn select_side(settings: &mut CreateGameSettings, side: UserRoleSelection) {
    settings.side = side;
}

/// Highlights the currently selected option button in a given button group
fn update_selected_option_button(
    commands: &mut Commands,
    selected: Entity,
    button_group: impl Iterator<Item = Entity>,
) {
    for entity in button_group {
        commands.entity(entity).insert(if entity == selected {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Normal
        });
    }
}

/// Callback for "Create Game".
/// This method sends the actual command to create a game to the server.
/// It also inserts the "PendingJoinSide" resource to the client backend. With this
/// resource, the client will wait for the server response with the created game, then join
/// the game.
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

/// dialog destructor
pub fn cleanup_create_dialog(mut commands: Commands, query: Query<Entity, With<Dialog>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<CreateGameSettings>();
}
