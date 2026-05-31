use bevy::feathers::controls::FeathersButton;
use bevy::feathers::display::{caption, label};
use bevy::feathers::theme::InheritableThemeTextColor;
use bevy::feathers::tokens;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use chess_core::protocol::messages::ClientMessage;

use crate::client::game::ActiveGame;
use crate::net::network::NetworkSend;
use crate::ui::theme;
use crate::ui::{Dialog, Overlay};

pub fn resign_dialog_scene() -> impl Scene {
    bsn! {
        Dialog
        Children [
            theme::default_dialog_node()
            theme::default_ui_shadow()
            BackgroundColor(theme::THEME_SURFACE)
            BorderColor::all(theme::THEME_BORDER)
            InheritableThemeTextColor(tokens::TEXT_MAIN)
            Children [
                @label("Resign game?")
                TextFont { font_size: FontSize::Rem(1.5) }
                TextLayout::justify(Justify::Center)
                --
                // line separator
                Node { width: percent(100), height: px(1) }
                BackgroundColor(theme::THEME_GOLD)
                --
                theme::default_button_row()
                Children [
                    @FeathersButton { @caption: bsn! { @caption("Cancel") } }
                    theme::default_button_size_150()
                    on(|_: On<Activate>, mut overlay: ResMut<NextState<Overlay>>| {
                        overlay.set(Overlay::None);
                    })
                    --
                    @FeathersButton { @caption: bsn! { @caption("Resign") } }
                    theme::default_button_size_150()
                    on(|_: On<Activate>,
                        game: Res<ActiveGame>,
                        mut overlay: ResMut<NextState<Overlay>>,
                        mut commands: Commands| {
                            commands.trigger(NetworkSend(ClientMessage::Resign(game.gid)));
                            overlay.set(Overlay::None);
                    })
                ]
            ]
        ]
    }
}
