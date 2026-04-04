use crate::ui::ButtonColors;
use crate::ui::COLOR_DARK2;

#[macro_export]
macro_rules! spawn_label {
    ($parent:ident, $text:expr, $font_size:expr, $color:expr) => {
        $parent.spawn((
            Text::new($text),
            TextFont {
                font_size: $font_size,
                ..default()
            },
            TextColor($color),
        ));
    };
    ($parent:ident, $text:expr, $font_size:expr) => {
        spawn_label!($parent, $text, $font_size, COLOR_DARK2);
    };
}

#[macro_export]
macro_rules! spawn_button {
    ($parent:ident, $text:expr, $action:expr, $colors:expr, $width:expr, $height:expr) => {
        $parent
            .spawn((
                Button,
                Node {
                    width: $width,
                    height: $height,
                    margin: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                $colors,
                $action,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new($text),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(COLOR_DARK),
                ));
            });
    };
    ($parent:ident, $text:expr, $action:expr, $colors:expr) => {
        spawn_button!(
            $parent,
            $text,
            $action,
            $colors,
            Val::Px(150.0),
            Val::Px(40.0)
        );
    };
    ($parent:ident, $text:expr, $action:expr) => {
        spawn_button!($parent, $text, $action, ButtonColors::default());
    };
}

#[macro_export]
macro_rules! spawn_dialog {
    ($commands:ident, $dialog_component:expr, $width:expr, $height:expr, $children:expr) => {
        $commands
            .spawn((
                Node {
                    // Root node needs this width/height, so the
                    // center alignment works. Actual size is given by the
                    // child container.
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                $dialog_component,
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            width: $width,
                            height: $height,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        BackgroundColor(COLOR_DARK2),
                        ZIndex(1),
                    ))
                    .with_children($children);
            });
    };
}
