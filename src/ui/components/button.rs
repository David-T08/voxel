use bevy::prelude::*;

pub fn spawn_text_button<'a>(parent: &'a mut ChildSpawnerCommands, label: &str) -> EntityCommands<'a> {
    let button = parent.spawn((
        Node {
            width: percent(25),
            height: px(28),
            
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                width: px(125),
                height: px(30),
                border: UiRect::all(px(3)),
                
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                
                ..default()
            },
            BorderColor::all(Color::srgb_u8(255, 140, 140)),
            BackgroundColor(Color::BLACK),
            children![(
                Text::new(label),
                TextColor(Color::WHITE)
            )]
        )]
    ));

    button
}