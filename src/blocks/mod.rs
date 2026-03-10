use bevy::prelude::*;
use jsonc_parser::parse_to_serde_value;
use std::{fs, sync::Arc};

pub mod registry;

use crate::{
    blocks::registry::BlockRegistryInner,
    textures::{BlockDefinitionAsset, BlockTextureRegistry, BlockTextureRegistryReady, atlas::BlockAtlas},
};
pub use registry::{BlockId, BlockRegistry};

pub const AIR_ID: BlockId = BlockId(0);

pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            load_block_definitions
                .run_if(resource_exists::<BlockTextureRegistryReady>)
                .run_if(resource_exists::<BlockAtlas>)
                .run_if(not(resource_exists::<BlockRegistryReady>)),
        );
    }
}

#[derive(Resource)]
pub struct BlockRegistryReady;

fn load_block_definitions(mut commands: Commands, tex_registry: Res<BlockTextureRegistry>, atlas: Res<BlockAtlas>) {
    let mut block_registry = BlockRegistryInner::default();
    block_registry.register_air();

    let asset_dir = fs::read_dir("./assets/blocks").unwrap();

    for entry in asset_dir {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("jsonc") {
            continue;
        }

        let stem = path.file_stem().unwrap().to_str().unwrap();
        let name = format!("core:{stem}");

        let text = fs::read_to_string(path.clone()).unwrap();
        let value = parse_to_serde_value(&text, &Default::default())
            .unwrap()
            .unwrap();
        let data = serde_json::from_value::<BlockDefinitionAsset>(value).unwrap();

        match block_registry.register_from_asset(name.clone(), data.textures, &tex_registry, &atlas) {
            Some(_) => println!(
                "Added blocks/{stem} to block registry, registered as: {:?}",
                block_registry.names.name_to_id(&name)
            ),
            None => eprintln!("Failed to add block {name} to registry"),
        };
    }

    block_registry.freeze();
    commands.insert_resource(BlockRegistry(Arc::new(block_registry)));
    commands.insert_resource(BlockRegistryReady);
}
