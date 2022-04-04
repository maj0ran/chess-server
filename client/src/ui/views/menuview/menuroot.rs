use crate::spawn_button;
use crate::state::{ClientState, MenuTab, Overlay, Screen};
use crate::ui::create_game_dialog::{
    cleanup_create_dialog, create_dialog_action_system, setup_create_dialog,
};
use crate::ui::gamelist_menu::{gamelist_menu_action_system, update_games_list};
use crate::ui::join_game_dialog::{
    cleanup_join_dialog, join_dialog_action_system, setup_join_dialog,
};
use crate::ui::views::menuview::analysismenu::analysis_menu::{
    cleanup_analysis_menu, setup_analysis_menu,
};
use crate::ui::views::menuview::gamemenu::gamelist_menu::{cleanup_menu, setup_gamelist_menu};
use crate::ui::ButtonColors;
use crate::ui::COLOR_DARK;
use bevy::prelude::*;

pub struct MenuRootPlugin;

#[derive(Component)]
pub struct MenuRootComponent;

#[derive(Component)]
pub struct MenuTabContainer;

#[derive(Component, Default)]
enum TabAction {
    #[default]
    GamesTab,
    AnalysisTab,
}

#[derive(Component)]
pub struct TabBar;

impl Plugin for MenuRootPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Menu), setup_menu_root)
            .add_systems(OnExit(Screen::Menu), (cleanup_menu_root, reset_menu_tab))
            .add_systems(Update, tab_action_system.run_if(in_state(Screen::Menu)))
            .add_systems(
                Update,
                (gamelist_menu_action_system
                    .run_if(in_state(Screen::Menu))
                    .run_if(in_state(MenuTab::Games)),),
            )
            .add_observer(update_games_list)
            // Create Game Dialog
            .add_systems(OnEnter(Overlay::CreateDialog), setup_create_dialog)
            .add_systems(OnExit(Overlay::CreateDialog), cleanup_create_dialog)
            .add_systems(
                Update,
                create_dialog_action_system.run_if(in_state(Overlay::CreateDialog)),
            )
            // Join Game Dialog
            .add_systems(OnEnter(Overlay::JoinDialog), setup_join_dialog)
            .add_systems(OnExit(Overlay::JoinDialog), cleanup_join_dialog)
            .add_systems(
                Update,
                join_dialog_action_system.run_if(in_state(Overlay::JoinDialog)),
            )
            .add_systems(
                OnEnter(MenuTab::Games),
                (
                    cleanup_analysis_menu,
                    setup_gamelist_menu.run_if(in_state(Screen::Menu)),
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(MenuTab::Analysis),
                (cleanup_menu, setup_analysis_menu).chain(),
            )
            .add_systems(
                OnEnter(MenuTab::None),
                (cleanup_menu, cleanup_analysis_menu),
            );
    }
}

pub fn setup_menu_root(
    mut commands: Commands,
    state: Res<ClientState>,
    mut next_tab: ResMut<NextState<MenuTab>>,
) {
    state.network.send(chess_core::ClientMessage::QueryGames);

    // set the current Tab manually so we trigger the logic for
    // the Games tab.
    next_tab.set(MenuTab::Games);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::flex(1.0)],
                grid_template_rows: vec![GridTrack::px(60.0), GridTrack::flex(1.0)],
                ..default()
            },
            MenuRootComponent,
            BackgroundColor(COLOR_DARK),
        ))
        .with_children(|p| {
            // Tab Bar
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    display: Display::Grid,
                    grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)],
                    grid_template_rows: vec![GridTrack::flex(1.0)],
                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    border: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
                TabBar,
                BorderColor::all(Color::WHITE),
            ))
            .with_children(|parent| {
                spawn_button!(
                    parent,
                    "Games",
                    TabAction::GamesTab,
                    ButtonColors::default(),
                    Val::Percent(80.0),
                    Val::Px(50.0)
                );
                spawn_button!(
                    parent,
                    "Analysis",
                    TabAction::AnalysisTab,
                    ButtonColors::default(),
                    Val::Percent(80.0),
                    Val::Px(50.0)
                );
            });

            // Content Container for the current tab. The tab views (GameList, Analysis)
            // will be inserted here. The logic is, that the views are querying for the
            // `MenuTabContainer` and will add themselves to it as a child.
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    display: Display::Grid,
                    justify_items: JustifyItems::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                MenuTabContainer,
            ));
        });
}

pub fn cleanup_menu_root(mut commands: Commands, query: Query<Entity, With<MenuRootComponent>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn reset_menu_tab(mut next_tab: ResMut<NextState<MenuTab>>) {
    next_tab.set(MenuTab::None);
}

fn tab_action_system(
    interaction_query: Query<(&Interaction, &TabAction), (Changed<Interaction>, With<Button>)>,
    mut next_tab: ResMut<NextState<MenuTab>>,
    current_tab: Res<State<MenuTab>>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match action {
                TabAction::GamesTab => {
                    if *current_tab.get() != MenuTab::Games {
                        next_tab.set(MenuTab::Games);
                    }
                }
                TabAction::AnalysisTab => {
                    if *current_tab.get() != MenuTab::Analysis {
                        next_tab.set(MenuTab::Analysis);
                    }
                }
            }
        }
    }
}
