use bevy::ui::AlignItems;

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
        spawn_label!($parent, $text, $font_size, COLOR_LIGHT);
    };
}

#[macro_export]
macro_rules! spawn_button {
    ($parent:ident, $text:expr, $action:expr, $colors:expr, $width:expr, $height:expr) => {
        $parent
            .spawn((
                Button,
                Interaction::default(),
                BackgroundColor($colors.normal),
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
            })
    };
    ($parent:ident, $text:expr, $action:expr, $colors:expr) => {
        spawn_button!(
            $parent,
            $text,
            $action,
            $colors,
            Val::Px(150.0),
            Val::Px(40.0)
        )
    };
    ($parent:ident, $text:expr, $action:expr) => {
        spawn_button!($parent, $text, $action, ButtonColors::default())
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
                    display: Display::Grid,
                    justify_items: JustifyItems::Center,
                    align_items: AlignItems::Center,
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
                            display: Display::Grid,
                            grid_template_columns: vec![GridTrack::flex(1.0)],
                            grid_auto_rows: vec![GridTrack::auto()],
                            row_gap: Val::Px(10.0),
                            justify_items: JustifyItems::Center,
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(20.0)),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(COLOR_MID),
                        ZIndex(1),
                    ))
                    .with_children($children);
            });
    };
}
