use bevy::feathers::controls::FeathersButton;
use bevy::feathers::theme::{InheritableThemeTextColor, ThemedText};
use bevy::feathers::tokens;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use chess_core::Promotion;

use crate::client::game::ActiveGame;
use crate::ui::theme::{WOOD_BORDER, WOOD_GOLD, WOOD_SURFACE};
use crate::ui::views::gameview::chessboard::assets::ChessAssets;
use crate::ui::views::gameview::chessboard::{PendingMove, RequestMove};
use crate::ui::{Dialog, Overlay};

#[derive(Component, Clone, Copy, Default)]
pub(crate) struct QueenPromotionImage;
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct RookPromotionImage;
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct BishopPromotionImage;
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct KnightPromotionImage;

pub fn promotion_dialog_scene() -> impl Scene {
    bsn! {
        Dialog
        Children[(
            Node { padding: UiRect::all(px(28)), flex_direction: FlexDirection::Column, row_gap: px(16), align_items: AlignItems::Center, border: UiRect::all(px(2)), border_radius: BorderRadius::all(px(10)) }
            BackgroundColor(WOOD_SURFACE)
            BorderColor::all(WOOD_BORDER)
            BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.6), px(0), px(6), px(2), px(10))
            InheritableThemeTextColor(tokens::TEXT_MAIN)
            Children[
                (Text("Promote Pawn") ThemedText TextFont { font_size: FontSize::Px(26.0) } TextLayout::justify(Justify::Center)),
                (Node { width: percent(100), height: px(1) } BackgroundColor(WOOD_GOLD)),
                (
                    Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::Center, column_gap: px(10) }
                    Children[
                        (@FeathersButton { @caption: bsn! { (Node { width: px(56), height: px(56) } ImageNode QueenPromotionImage) } } Node { width: px(110), height: px(72) } on(|_: On<Activate>, pending: Res<PendingMove>, commands: Commands, next_overlay: ResMut<NextState<Overlay>>| {
                            promote(Promotion::Queen, pending, commands, next_overlay);
                        })),
                        (@FeathersButton { @caption: bsn! { (Node { width: px(56), height: px(56) } ImageNode RookPromotionImage) } } Node { width: px(110), height: px(72) } on(|_: On<Activate>, pending: Res<PendingMove>, commands: Commands, next_overlay: ResMut<NextState<Overlay>>| {
                            promote(Promotion::Rook, pending, commands, next_overlay);
                        })),
                        (@FeathersButton { @caption: bsn! { (Node { width: px(56), height: px(56) } ImageNode BishopPromotionImage) } } Node { width: px(110), height: px(72) } on(|_: On<Activate>, pending: Res<PendingMove>, commands: Commands, next_overlay: ResMut<NextState<Overlay>>| {
                            promote(Promotion::Bishop, pending, commands, next_overlay);
                        })),
                        (@FeathersButton { @caption: bsn! { (Node { width: px(56), height: px(56) } ImageNode KnightPromotionImage) } } Node { width: px(110), height: px(72) } on(|_: On<Activate>, pending: Res<PendingMove>, commands: Commands, next_overlay: ResMut<NextState<Overlay>>| {
                            promote(Promotion::Knight, pending, commands, next_overlay);
                        }))
                    ]
                )
            ]
        )]
    }
}

pub(crate) fn initialize_promotion_images(
    assets: Res<ChessAssets>,
    game: Res<ActiveGame>,
    pending: Res<PendingMove>,
    mut images: Query<
        (
            &mut ImageNode,
            Option<&QueenPromotionImage>,
            Option<&RookPromotionImage>,
            Option<&BishopPromotionImage>,
            Option<&KnightPromotionImage>,
        ),
        Or<(
            With<QueenPromotionImage>,
            With<RookPromotionImage>,
            With<BishopPromotionImage>,
            With<KnightPromotionImage>,
        )>,
    >,
) {
    let pawn = game
        .internal_board
        .get(&pending.src)
        .copied()
        .unwrap_or(if pending.dst.ends_with('8') { 'P' } else { 'p' });
    let (queen, rook, bishop, knight) = if pawn.is_uppercase() {
        ('Q', 'R', 'B', 'N')
    } else {
        ('q', 'r', 'b', 'n')
    };

    for (mut image, queen_marker, rook_marker, bishop_marker, knight_marker) in images.iter_mut() {
        let piece = if queen_marker.is_some() {
            queen
        } else if rook_marker.is_some() {
            rook
        } else if bishop_marker.is_some() {
            bishop
        } else if knight_marker.is_some() {
            knight
        } else {
            continue;
        };

        image.image = assets.get(piece).cloned().expect("missing promotion asset");
    }
}

fn promote(
    piece: Promotion,
    pending: Res<PendingMove>,
    mut commands: Commands,
    mut next_overlay: ResMut<NextState<Overlay>>,
) {
    commands.trigger(RequestMove {
        source: pending.src.clone(),
        destination: pending.dst.clone(),
        promotion: Some(piece),
    });
    commands.remove_resource::<PendingMove>();
    next_overlay.set(Overlay::None);
}

pub fn despawn_promotion_dialog(mut commands: Commands, query: Query<Entity, With<Dialog>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
