use std::collections::HashMap;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::player::input::PlayerInput;

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app 
            .add_systems(Startup, init)
            .add_systems(
                Update, 
                (
                    update_position,
                    set_mouse_lock
                )   
            );
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CursorIcon {
    Crosshair,
}

#[derive(Component)]
pub struct PlayerCursorMarker;

impl CursorIcon {
    pub fn file_name(&self) -> &'static str {
        match *self {
            CursorIcon::Crosshair => "crosshair.png",
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct CursorLookup {
    pub cursors: HashMap<CursorIcon, Handle<Image>>
} 

#[derive(Resource)]
pub struct PlayerCursor {
    pub locked: bool,
    pub visible: bool,
    pub icon: CursorIcon,
    pub position: Vec2,
}

pub fn init(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    let mut lookup = CursorLookup::default();
    
    for cursor in [CursorIcon::Crosshair] {
        let handle: Handle<Image> = asset_server.load(format!("ui/{}", cursor.file_name()));
        
        lookup.insert(cursor, handle);
    }
    
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        
        children![(
            
        )]
    ))
    
    commands.insert_resource(lookup);
    commands.insert_resource(PlayerCursor {
        locked: true,
        icon: CursorIcon::Crosshair,
        visible: true,
        position: Vec2::ZERO,
    });
}

pub fn update_position(mut cursor: ResMut<PlayerCursor>, window: Single<&Window, With<PrimaryWindow>>) {
    let mut position = window.cursor_position()
        .unwrap_or_else(|| {
            window.size() / 2.0
        });
    
    if cursor.locked && cursor.icon == CursorIcon::Crosshair {
        position = window.size() / 2.0;
    }
    
    cursor.position = position;
}

pub fn set_mouse_lock(mut cursor: ResMut<PlayerCursor>, input: Res<PlayerInput>) {
    match input.mouse.cursor_unlocked {
        false => {
            cursor.icon = CursorIcon::Crosshair;
        }

        true => {
            // cursor.icon = CursorIco
            
        }
    };
}

pub fn render_cursor(
    cursor: Res<PlayerCursor>
)