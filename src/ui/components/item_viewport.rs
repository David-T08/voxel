use bevy::prelude::*;
use crate::{blocks::BlockId, registry_base::LookupRegistry};

pub const MAX_TEXTURE_CACHE: usize = 1024;

#[derive(Component)]
pub struct ItemBakerCamera;

#[derive(Component)]
pub struct ItemBakerRoot;

#[derive(Resource, Deref, DerefMut)]
pub struct IconCache(pub LookupRegistry<BlockId, Handle<Image>>);

fn setup_cameras(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>
) {
    
}

pub fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>
) {
    setup_cameras(commands, images);
}

impl IconCache {
    pub fn from_id(&self, id: BlockId) -> Option<&Handle<Image>> {
        let handle = self.get(id).cloned();
        
        handle
    }
}