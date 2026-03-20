use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::{blocks::BlockId, player::input::PlayerInput, registry_base::RegistryId, ui::components::block_viewport::BlockIconCache};

const ICON_SIZE: usize = 64;
const ICON_PADDING: usize = 32;


#[derive(Component)]
pub struct HotbarRoot;

#[derive(Component, Deref, DerefMut)]
pub struct HotbarIcon(pub BlockId);

#[derive(Component)]
pub struct HotbarSlot {
    pub index: usize,
    pub block: BlockId
}

#[derive(Component)]
pub struct SelectedHotbarSlot;

pub struct CoreHotbarPlugin;
impl Plugin for CoreHotbarPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, init)
            .add_systems(Update, (
                handle_input,
                populate_hotbar_icons,
                run_highlighting,
                handle_hotbar_keyboard_input,
                handle_hotbar_scroll_input
            ));
    }
}

pub fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    let image: Handle<Image> = asset_server.load("ui/hotbar-icon.png");

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
                    width: px((ICON_PADDING + 6 + ICON_SIZE) * 9 - 6),
                    height: px(ICON_SIZE + ICON_PADDING),
                    flex_direction: FlexDirection::Row,
                    column_gap: px(6),
                    ..default()
                },
                HotbarRoot,
            ))
            .with_children(|p| {
                for i in 0..9 {
                    create_slot(p, image.clone(), i)
                }
            });
        });
}

pub fn handle_input(
    input: Res<PlayerInput>,
) {
    
}

pub fn create_slot<'a>(parent: &'a mut ChildSpawnerCommands, image: Handle<Image>, index: usize) {
    let mut slot = parent.spawn((
        Node {
            width: px(ICON_SIZE + ICON_PADDING),
            height: px(ICON_SIZE + ICON_PADDING),
            
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        
        ImageNode {
            image,
            image_mode: NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(4.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 4.
            }),
            ..default()
        },
        
        HotbarSlot {index, block: BlockId::from_index(index + 1)}
    ));
    
    if index == 0 {
        slot.insert(SelectedHotbarSlot);
    }
    
    slot.with_child((
        Node {
            width: px(ICON_SIZE),
            height: px(ICON_SIZE),
            ..default()
        },
        
        ImageNode::default(),
        HotbarIcon(BlockId::from_index(index + 1))
    ));
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
        
        // node.width = px(width);
        // node.height = px(height);
    }
}

pub fn run_highlighting(
    mut selected: Query<&mut ImageNode, With<SelectedHotbarSlot>>,
    mut unselected: Query<&mut ImageNode, (With<HotbarSlot>, Without<SelectedHotbarSlot>)>,
) {
    for mut image in &mut selected {
        image.color = Color::srgb(1.15, 1.1, 1.);
    }

    for mut image in &mut unselected {
        image.color = Color::WHITE;
    }
}

pub fn handle_hotbar_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    selected: Query<Entity, With<SelectedHotbarSlot>>,
    slots: Query<(Entity, &HotbarSlot)>,
) {
    let pressed_index = if keys.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(2)
    } else if keys.just_pressed(KeyCode::Digit4) {
        Some(3)
    } else if keys.just_pressed(KeyCode::Digit5) {
        Some(4)
    } else if keys.just_pressed(KeyCode::Digit6) {
        Some(5)
    } else if keys.just_pressed(KeyCode::Digit7) {
        Some(6)
    } else if keys.just_pressed(KeyCode::Digit8) {
        Some(7)
    } else if keys.just_pressed(KeyCode::Digit9) {
        Some(8)
    } else {
        None
    };

    let Some(index) = pressed_index else {
        return;
    };

    if let Ok(current) = selected.single() {
        commands.entity(current).remove::<SelectedHotbarSlot>();
    }

    for (entity, slot) in &slots {
        if slot.index == index {
            commands.entity(entity).insert(SelectedHotbarSlot);
            break;
        }
    }
}

pub fn handle_hotbar_scroll_input(
    mut scroll_evr: MessageReader<MouseWheel>,
    mut commands: Commands,
    selected: Single<(Entity, &HotbarSlot), With<SelectedHotbarSlot>>,
    slots: Query<(Entity, &HotbarSlot)>,
) {

    let (current_entity, current_slot) = selected.into_inner();

    let mut delta = 0i32;

    for ev in scroll_evr.read() {
        if ev.y > 0.0 {
            delta -= 1; 
        } else if ev.y < 0.0 {
            delta += 1;
        }
    }

    if delta == 0 {
        return;
    }

    let current = current_slot.index as i32;
    let next = (current + delta).rem_euclid(9) as usize;

    commands.entity(current_entity).remove::<SelectedHotbarSlot>();

    for (entity, slot) in &slots {
        if slot.index == next {
            commands.entity(entity).insert(SelectedHotbarSlot);
            break;
        }
    }
}