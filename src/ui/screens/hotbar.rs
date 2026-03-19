use bevy::prelude::*;

use crate::{blocks::BlockId, ui::components::{
    block_viewport::BlockIconCache, button::spawn_text_button
}};

#[derive(Component)]
pub struct HotbarRoot;

#[derive(Component, Deref, DerefMut)]
pub struct HotbarIcon(pub BlockId);

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
                p.spawn((
                    Node {
                        width: px(64),
                        height: px(64),
                        ..default()
                    },
                    ImageNode::default(),
                    HotbarIcon(BlockId(1))
                ));
                
                p.spawn((
                    Node {
                        width: px(64),
                        height: px(64),
                        ..default()
                    },
                    ImageNode::default(),
                    HotbarIcon(BlockId(2))
                ));
            });
        });
}

pub fn populate_hotbar_icons(
    icons: Option<Res<BlockIconCache>>,
    mut q: Query<(&HotbarIcon, &mut ImageNode, &mut Node)>,
) {
    let Some(icons) = icons else {
        return; // atlas not ready yet
    };
    
    if icons.icons.entries.len() == 0 {
        return
    }

    for (hotbar_icon, mut image_node, mut node) in &mut q {
        let Some(icon) = icons.get(hotbar_icon.0) else {
            continue;
        };

        let width = icon[2] - icon[0];
        let height = icon[3] - icon[1];

        image_node.image = icons.atlas.clone();
        image_node.rect = Some(Rect {
            min: Vec2::new(icon[0], icon[1]),
            max: Vec2::new(icon[2], icon[3]),
        });
        
        node.width = px(width);
        node.height = px(height);
    }
}