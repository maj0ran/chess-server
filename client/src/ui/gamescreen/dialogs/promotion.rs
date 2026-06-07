use bevy::feathers::controls::FeathersButton;
use bevy::feathers::display::label;
use bevy::feathers::theme::InheritableThemeTextColor;
use bevy::feathers::tokens;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use chess_core::Promotion;

use crate::backend::ActiveGame;
use crate::ui::gamescreen::chessboard::{ChessAssets, PendingMove, RequestMove};
use crate::ui::theme;
use crate::ui::{Dialog, Overlay};

#[derive(Component, Clone, Copy)]
pub(crate) struct PromotionImage(pub Promotion);

impl Default for PromotionImage {
    fn default() -> Self {
        Self(Promotion::Queen)
    }
}

/// Macro to insert a piece image to the promotion buttons.
macro_rules! promotion_image {
    ($piece:ident) => {
        bsn! {
            Node { width: px(56), height: px(56) }
            ImageNode
            PromotionImage(Promotion::$piece)
        }
    };
}

pub fn promotion_dialog_scene() -> impl Scene {
    bsn! {
        Dialog
        Children [
            theme::default_dialog_node()
            theme::default_ui_shadow()
            BackgroundColor(theme::THEME_SURFACE)
            BorderColor::all(theme::THEME_BORDER)
            InheritableThemeTextColor(tokens::TEXT_MAIN)
            Children [
                @label("Promote Pawn")
                TextFont { font_size: FontSize::Rem(1.5) }
                TextLayout::justify(Justify::Center)
                --
                // line separator
                Node { width: percent(100), height: px(1) }
                BackgroundColor(theme::THEME_GOLD)
                --
                theme::default_button_row()
                Children [
                    @FeathersButton {
                        @caption: promotion_image!(Queen)
                    }
                    theme::default_button_icon()
                    on(|_: On<Activate>,
                        pending: Res<PendingMove>,
                        commands: Commands,
                        next_overlay: ResMut<NextState<Overlay>>| {
                            promote(Promotion::Queen, pending, commands, next_overlay);
                    })
                    --
                    @FeathersButton {
                        @caption: promotion_image!(Rook)
                    }
                    theme::default_button_icon()
                    on(|_: On<Activate>,
                        pending: Res<PendingMove>,
                        commands: Commands,
                        next_overlay: ResMut<NextState<Overlay>>| {
                            promote(Promotion::Rook, pending, commands, next_overlay);
                    })
                    --
                    @FeathersButton {
                        @caption: promotion_image!(Bishop)
                    }
                    theme::default_button_icon()
                    on(|_: On<Activate>,
                        pending: Res<PendingMove>,
                        commands: Commands,
                        next_overlay: ResMut<NextState<Overlay>>| {
                            promote(Promotion::Bishop, pending, commands, next_overlay);
                    })
                    --
                    @FeathersButton {
                        @caption: promotion_image!(Knight)
                    }
                    theme::default_button_icon()
                    on(|_: On<Activate>,
                        pending: Res<PendingMove>,
                        commands: Commands,
                        next_overlay: ResMut<NextState<Overlay>>| {
                            promote(Promotion::Knight, pending, commands, next_overlay);
                    })
                ]
            ]
        ]
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
