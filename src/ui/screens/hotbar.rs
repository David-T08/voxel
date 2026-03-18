use bevy::prelude::*;

use crate::ui::components::button::spawn_text_button;

#[derive(Component)]
pub struct HotbarRoot;

pub fn init(mut commands: Commands) {
    commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::End,
            padding: UiRect::bottom(px(12)),
            ..default()
        })
        .with_children(|p| {
            p.spawn((
                Node {
                    width: px((64 + 6) * 10),
                    height: px(64),
                    flex_direction: FlexDirection::Row,
                    column_gap: px(6),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                HotbarRoot,
            ))
            .with_children(|p| {
                spawn_text_button(p, "test 1");
                spawn_text_button(p, "test 2");
                spawn_text_button(p, "test 3");
            });
        });
}