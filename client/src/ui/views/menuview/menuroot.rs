use crate::client::network::NetworkSend;
use crate::ui::views::menuview::analysismenu::analysis_menu::{
    cleanup_analysis_menu, setup_analysis_menu,
};
use crate::ui::views::menuview::gamemenu::dialogs::create_game_dialog::{
    cleanup_create_dialog, create_dialog_action_system, setup_create_dialog,
};
use crate::ui::views::menuview::gamemenu::dialogs::join_game_dialog::{
    cleanup_join_dialog, join_dialog_action_system, setup_join_dialog,
};
use crate::ui::views::menuview::gamemenu::gamelist_menu::{
    cleanup_gamelist_menu, gamelist_menu_action_system, setup_gamelist_menu, update_games_list,
};
use crate::ui::views::menuview::puzzlemenu::puzzle_menu::{cleanup_puzzle_menu, setup_puzzle_menu};

use crate::ui::{MenuTab, Overlay, Screen};
use bevy::prelude::*;
use bevy_flair::prelude::*;
use chess_core::protocol::messages::ClientMessage;

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
    PuzzleTab,
}

#[derive(Component)]
pub struct TabBar;

impl Plugin for MenuRootPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Menu), setup_menu_root)
            .add_systems(OnExit(Screen::Menu), cleanup_menu_root)
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
            .add_systems(OnEnter(MenuTab::Games), setup_gamelist_menu)
            .add_systems(OnExit(MenuTab::Games), cleanup_gamelist_menu)
            .add_systems(OnExit(MenuTab::Analysis), cleanup_analysis_menu)
            .add_systems(OnExit(MenuTab::Puzzle), cleanup_puzzle_menu)
            .add_systems(OnEnter(MenuTab::Analysis), setup_analysis_menu)
            .add_systems(OnEnter(MenuTab::Puzzle), setup_puzzle_menu);
    }
}

pub fn setup_menu_root(
    mut commands: Commands,
    mut next_tab: ResMut<NextState<MenuTab>>,
    asset_server: Res<AssetServer>,
) {
    commands.trigger(NetworkSend(ClientMessage::QueryGames));

    // set the current Tab manually so we trigger the logic for
    // the Games tab.
    next_tab.set(MenuTab::Games);

    commands.spawn((
        Node::default(),
        NodeStyleSheet::new(asset_server.load("style.css")),
        MenuRootComponent,
        ClassList::new("menu-root"),
        children![
            (
                Node::default(),
                ClassList::new("label-large title-bar"),
                children![Text::new("Schach!")],
            ),
            (
                Node::default(),
                TabBar,
                ClassList::new("tab-bar"),
                children![
                    (
                        Button,
                        Interaction::default(),
                        ClassList::new("tab-button"),
                        TabAction::GamesTab,
                        children![Text::new("Play")],
                    ),
                    (
                        Button,
                        Interaction::default(),
                        ClassList::new("tab-button"),
                        TabAction::AnalysisTab,
                        children![Text::new("Analysis")],
                    ),
                    (
                        Button,
                        Interaction::default(),
                        ClassList::new("tab-button"),
                        TabAction::PuzzleTab,
                        children![Text::new("Puzzles")],
                    )
                ],
            ),
            // container where the tab content is rendered into. (Play-tab, Analysis-tab)
            (
                Node::default(),
                MenuTabContainer,
                ClassList::new("content-container"),
            )
        ],
    ));
}

pub fn cleanup_menu_root(
    mut commands: Commands,
    query: Query<Entity, With<MenuRootComponent>>,
    mut next_tab: ResMut<NextState<MenuTab>>,
) {
    next_tab.set(MenuTab::None);
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
                TabAction::PuzzleTab => {
                    if *current_tab.get() != MenuTab::Puzzle {
                        next_tab.set(MenuTab::Puzzle);
                    }
                }
            }
        }
    }
}
