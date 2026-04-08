use crate::ui::views::menuview::menuroot::MenuTabContainer;
use crate::ui::views::menuview::MenuTabComponent;
use bevy::prelude::*;
use bevy_flair::prelude::*;

#[derive(Component)]
pub struct PuzzleMenuComponent;

#[derive(Component)]
pub enum PuzzleAction {
    Todo,
}

pub fn setup_puzzle_menu(
    mut commands: Commands,
    container_query: Query<Entity, With<MenuTabContainer>>,
    asset_server: Res<AssetServer>,
) {
    let container = container_query.single();

    let menu_node = commands
        .spawn((
            Node::default(),
            NodeStyleSheet::new(asset_server.load("style.css")),
            PuzzleMenuComponent,
            MenuTabComponent,
            ClassList::new("menu"),
            children![
                (Text::new("Puzzle Menu"), ClassList::new("label-large")),
                (
                    Text::new("Under construction..."),
                    ClassList::new("label-small")
                )
            ],
        ))
        .id();

    if let Ok(container) = container {
        commands.entity(container).add_child(menu_node);
    }
}

pub fn cleanup_puzzle_menu(
    mut commands: Commands,
    query: Query<Entity, With<PuzzleMenuComponent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
