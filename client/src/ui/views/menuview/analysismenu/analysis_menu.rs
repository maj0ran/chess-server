use crate::ui::views::menuview::menuroot::MenuTabContainer;
use crate::ui::views::menuview::MenuTabComponent;
use crate::ui::ButtonColors;
use crate::COLOR_DARK;
use bevy::color::Color;
use bevy::prelude::*;

#[derive(Component)]
pub struct AnalysisMenuComponent;

#[derive(Component)]
pub enum AnalysisAction {
    Todo,
}

pub fn setup_analysis_menu(
    mut commands: Commands,
    container_query: Query<Entity, With<MenuTabContainer>>,
    existing_query: Query<Entity, With<AnalysisMenuComponent>>,
) {
    // Prevent duplicate menus
    if !existing_query.is_empty() {
        return;
    }

    let container = container_query.single().ok();

    let menu_node = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            AnalysisMenuComponent,
            MenuTabComponent,
        ))
        .with_children(|p| {
            spawn_label!(p, "Under Construction", 40.0, Color::WHITE);
            spawn_button!(
                p,
                "Analysis Button",
                AnalysisAction::Todo,
                ButtonColors::default(),
                Val::Px(200.0),
                Val::Px(50.0)
            );
        })
        .id();

    if let Some(container) = container {
        commands.entity(container).add_child(menu_node);
    }
}

pub fn cleanup_analysis_menu(
    mut commands: Commands,
    query: Query<Entity, With<AnalysisMenuComponent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
