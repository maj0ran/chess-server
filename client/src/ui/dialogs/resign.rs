use bevy::feathers::controls::FeathersButton;
use bevy::feathers::theme::{InheritableThemeTextColor, ThemedText};
use bevy::feathers::tokens;
use bevy::prelude::*;
use chess_core::protocol::messages::ClientMessage;

use crate::client::game::ActiveGame;
use crate::net::network::NetworkSend;
use crate::ui::theme::{WOOD_BORDER, WOOD_GOLD, WOOD_SURFACE};
use crate::ui::{Dialog, Overlay};

pub fn resign_dialog_scene() -> impl Scene {
    bsn! {
        Dialog
        Children[(
            Node { width: px(420), padding: UiRect::all(px(28)), flex_direction: FlexDirection::Column, row_gap: px(16), align_items: AlignItems::Center, border: UiRect::all(px(2)), border_radius: BorderRadius::all(px(10)) }
            BackgroundColor(WOOD_SURFACE)
            BorderColor::all(WOOD_BORDER)
            BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.6), px(0), px(6), px(2), px(10))
            InheritableThemeTextColor(tokens::TEXT_MAIN)
            Children[
                (Text("Resign game?") ThemedText TextFont { font_size: FontSize::Px(26.0) } TextLayout::justify(Justify::Center)),
                (Node { width: percent(100), height: px(1) } BackgroundColor(WOOD_GOLD)),
                (
                    Node { width: percent(100), flex_direction: FlexDirection::Row, justify_content: JustifyContent::Center, column_gap: px(12) }
                    Children[
                        (@FeathersButton { @caption: bsn! { Text("Cancel") ThemedText } } Node { width: px(150), height: px(50) } on(|_: On<bevy::ui_widgets::Activate>, mut overlay: ResMut<NextState<Overlay>>| overlay.set(Overlay::None))),
                        (@FeathersButton { @caption: bsn! { Text("Resign") ThemedText } } Node { width: px(150), height: px(50) } on(|_: On<bevy::ui_widgets::Activate>, game: Res<ActiveGame>, mut overlay: ResMut<NextState<Overlay>>, mut commands: Commands| {
                            commands.trigger(NetworkSend(ClientMessage::Resign(game.gid)));
                            overlay.set(Overlay::None);
                        }))
                    ]
                )
            ]
        )]
    }
}
