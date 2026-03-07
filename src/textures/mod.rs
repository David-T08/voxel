use bevy::prelude::*;
use std::fs;

pub mod registry;
pub mod atlas;

use atlas::{UnbuiltBlockAtlas};
use registry::{BlockTextureRegistry};

use crate::textures::atlas::BlockAtlas;

pub struct TexturePlugin;
impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnbuiltBlockAtlas>();
        app.init_resource::<BlockTextureRegistry>();
        
        app.add_systems(Startup, load_block_textures);
        app.add_systems(Update, build_block_atlas.run_if(not(resource_exists::<BlockAtlas>)));
    }
}

fn load_block_textures(
    asset_server: Res<AssetServer>,
    mut atlas: ResMut<UnbuiltBlockAtlas>,
    mut reg: ResMut<BlockTextureRegistry>
) {
    let asset_dir = fs::read_dir("./assets/textures").unwrap();
    
    for entry in asset_dir {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) != Some("png") {
            continue;
        }
        
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let key = format!("core:{stem}");
        
        let asset_path = format!("textures/{stem}.png");
        
        let id = reg.register(key.clone()).unwrap();
        let handle: Handle<Image> = asset_server.load(&asset_path);
        
        match atlas.insert(id, handle) {
            Ok(()) => println!("Added {asset_path} to block atlas, registered as: {:?}, id->name={:?}", reg.name_to_id(&key), reg.id_to_name(id)),
            Err(e) => eprintln!("Failed to add {asset_path} ({id}): {:?}", e)
        };
    }
    
    reg.freeze();
    atlas.mark_ready();
}

fn build_block_atlas(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<UnbuiltBlockAtlas>
) {
    let status = atlas.ready_status(&assets, &images);
    if !status.ready {
        return;
    }
    
    let unbuilt = std::mem::take(&mut atlas.0);
    
    match unbuilt.build(&assets, &mut images) {
        Ok(built) => {
            commands.insert_resource(BlockAtlas(built));
        }
        
        Err((errs, unbuilt)) => {
            atlas.0 = unbuilt;
            error!("Failed to build block atlas: {:?}", errs);
        }
    }
}