use crate::backend::client::Overlay;
use crate::ui::views::gameview::chessboard::assets::ChessAssets;
use crate::ui::views::gameview::chessboard::{PendingMove, RequestMove};
use bevy::prelude::*;
use bevy_flair::prelude::{ClassList, NodeStyleSheet};
use chess_core::Promotion;

#[derive(Component)]
pub struct PromotionDialogComponent;

#[derive(Component)]
pub enum PromotionAction {
    Queen,
    Knight,
    Rook,
    Bishop,
    Cancel,
}

pub fn spawn_promotion_dialog(
    mut commands: Commands,
    assets: Res<ChessAssets>,
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
                                    if let Some(texture) = assets.get(c) {
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

pub fn promotion_action_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &PromotionAction),
        (Changed<Interaction>, With<Button>),
    >,
    pending_move: Option<Res<PendingMove>>,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    let mov = pending_move.unwrap();
    for (interaction, promotion_request) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let promotion = match promotion_request {
                PromotionAction::Queen => Some(Promotion::Queen),
                PromotionAction::Rook => Some(Promotion::Rook),
                PromotionAction::Bishop => Some(Promotion::Bishop),
                PromotionAction::Knight => Some(Promotion::Knight),
                PromotionAction::Cancel => None,
            };

            commands.trigger(RequestMove {
                source: mov.src.clone(),
                destination: mov.dst.clone(),
                promotion,
            });

            commands.remove_resource::<PendingMove>();
            next_overlay.set(Overlay::None);
            break;
        }
    }
}

pub fn despawn_promotion_dialog(
    mut commands: Commands,
    query: Query<Entity, With<PromotionDialogComponent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
