use bevy::feathers::controls::FeathersButton;
use bevy::feathers::theme::{InheritableThemeTextColor, ThemedText};
use bevy::feathers::tokens;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use chess_core::Promotion;

use crate::client::game::ActiveGame;
use crate::ui::ingame::chessboard::assets::ChessAssets;
use crate::ui::ingame::chessboard::{PendingMove, RequestMove};
use crate::ui::theme::{WOOD_BORDER, WOOD_GOLD, WOOD_SURFACE};
use crate::ui::{Dialog, Overlay};

#[derive(Component, Clone, Copy)]
pub(crate) struct PromotionImage(pub Promotion);

impl Default for PromotionImage {
    fn default() -> Self {
        Self(Promotion::Queen)
    }
}

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
                        (@FeathersButton { @caption: bsn! { (Node { width: px(56), height: px(56) } ImageNode PromotionImage(Promotion::Queen)) } } Node { width: px(110), height: px(72) } on(|_: On<Activate>, pending: Res<PendingMove>, commands: Commands, next_overlay: ResMut<NextState<Overlay>>| {
                            promote(Promotion::Queen, pending, commands, next_overlay);
                        })),
                        (@FeathersButton { @caption: bsn! { (Node { width: px(56), height: px(56) } ImageNode PromotionImage(Promotion::Rook)) } } Node { width: px(110), height: px(72) } on(|_: On<Activate>, pending: Res<PendingMove>, commands: Commands, next_overlay: ResMut<NextState<Overlay>>| {
                            promote(Promotion::Rook, pending, commands, next_overlay);
                        })),
                        (@FeathersButton { @caption: bsn! { (Node { width: px(56), height: px(56) } ImageNode PromotionImage(Promotion::Bishop)) } } Node { width: px(110), height: px(72) } on(|_: On<Activate>, pending: Res<PendingMove>, commands: Commands, next_overlay: ResMut<NextState<Overlay>>| {
                            promote(Promotion::Bishop, pending, commands, next_overlay);
                        })),
                        (@FeathersButton { @caption: bsn! { (Node { width: px(56), height: px(56) } ImageNode PromotionImage(Promotion::Knight)) } } Node { width: px(110), height: px(72) } on(|_: On<Activate>, pending: Res<PendingMove>, commands: Commands, next_overlay: ResMut<NextState<Overlay>>| {
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
    mut images: Query<(&mut ImageNode, &PromotionImage)>,
) {
    let pawn = game
        .internal_board
        .get(&pending.src)
        .copied()
        .unwrap_or(if pending.dst.ends_with('8') { 'P' } else { 'p' });
    let piece = |promotion| match (pawn.is_uppercase(), promotion) {
        (true, Promotion::Queen) => 'Q',
        (true, Promotion::Rook) => 'R',
        (true, Promotion::Bishop) => 'B',
        (true, Promotion::Knight) => 'N',
        (false, Promotion::Queen) => 'q',
        (false, Promotion::Rook) => 'r',
        (false, Promotion::Bishop) => 'b',
        (false, Promotion::Knight) => 'n',
    };

    for (mut image, promotion_image) in &mut images {
        let piece = piece(promotion_image.0);

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
