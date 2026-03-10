use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

pub mod atlas;
pub mod registry;

use atlas::{BlockAtlas, UnbuiltBlockAtlas};
pub use registry::{BlockTextureId, BlockTextureRegistry};

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Face {
    Top = 0,
    Bottom = 1,
    Front = 2,
    Back = 3,
    Left = 4,
    Right = 5,
}

#[derive(Resource, Default)]
pub struct BlockTextureRegistryReady;

#[derive(Deserialize)]
pub struct BlockDefinitionAsset {
    pub textures: BlockTexturesAsset,
}

#[derive(Deserialize, Clone)]
pub struct BlockTexturesAsset {
    pub top: Option<String>,
    pub bottom: Option<String>,

    pub front: Option<String>,
    pub back: Option<String>,

    pub left: Option<String>,
    pub right: Option<String>,

    pub side: Option<String>,
    pub all: Option<String>,
}

impl BlockTexturesAsset {
    pub fn resolve(self, reg: &BlockTextureRegistry) -> Result<BlockTextures, String> {
        fn get_tex(reg: &BlockTextureRegistry, name: &str) -> Result<BlockTextureId, String> {
            reg.name_to_id(name)
                .ok_or_else(|| format!("unknown block texture: {name}"))
        }

        if let Some(all) = self.all {
            return Ok(BlockTextures::from_all(get_tex(reg, &all)?));
        }

        if let Some(side) = self.side {
            let top = self
                .top
                .as_deref()
                .ok_or_else(|| "missing top texture".to_string())?;
            let bottom = self
                .bottom
                .as_deref()
                .ok_or_else(|| "missing bottom texture".to_string())?;

            return Ok(BlockTextures::from_top_bottom_side(
                get_tex(reg, top)?,
                get_tex(reg, bottom)?,
                get_tex(reg, &side)?,
            ));
        }

        Ok(BlockTextures::new(
            get_tex(
                reg,
                self.top
                    .as_deref()
                    .ok_or_else(|| "missing top texture".to_string())?,
            )?,
            get_tex(
                reg,
                self.bottom
                    .as_deref()
                    .ok_or_else(|| "missing bottom texture".to_string())?,
            )?,
            get_tex(
                reg,
                self.front
                    .as_deref()
                    .ok_or_else(|| "missing front texture".to_string())?,
            )?,
            get_tex(
                reg,
                self.back
                    .as_deref()
                    .ok_or_else(|| "missing back texture".to_string())?,
            )?,
            get_tex(
                reg,
                self.left
                    .as_deref()
                    .ok_or_else(|| "missing left texture".to_string())?,
            )?,
            get_tex(
                reg,
                self.right
                    .as_deref()
                    .ok_or_else(|| "missing right texture".to_string())?,
            )?,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct BlockTextures {
    pub top: BlockTextureId,
    pub bottom: BlockTextureId,

    pub front: BlockTextureId,
    pub back: BlockTextureId,

    pub left: BlockTextureId,
    pub right: BlockTextureId,
}

impl BlockTextures {
    pub fn new(
        top: BlockTextureId,
        bottom: BlockTextureId,
        front: BlockTextureId,
        back: BlockTextureId,
        left: BlockTextureId,
        right: BlockTextureId,
    ) -> Self {
        Self {
            top,
            bottom,

            front,
            back,

            left,
            right,
        }
    }

    pub fn from_top_bottom_side(
        top: BlockTextureId,
        bottom: BlockTextureId,
        side: BlockTextureId,
    ) -> Self {
        Self {
            top,
            bottom,

            front: side,
            back: side,

            left: side,
            right: side,
        }
    }

    pub fn from_all(all: BlockTextureId) -> Self {
        Self {
            top: all,
            bottom: all,

            front: all,
            back: all,

            left: all,
            right: all,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = BlockTextureId> {
        [
            self.top,
            self.bottom,
            self.front,
            self.back,
            self.left,
            self.right,
        ]
        .into_iter()
    }

    pub fn get_uvs(&self, atlas: &BlockAtlas) -> [[[f32; 2]; 4]; 6] {
        [
            atlas.face_uvs(self.top).unwrap(),
            atlas.face_uvs(self.bottom).unwrap(),
            atlas.face_uvs(self.front).unwrap(),
            atlas.face_uvs(self.back).unwrap(),
            atlas.face_uvs(self.left).unwrap(),
            atlas.face_uvs(self.right).unwrap(),
        ]
    }
}

pub struct TexturePlugin;
impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnbuiltBlockAtlas>();
        app.init_resource::<BlockTextureRegistry>();

        app.add_systems(Startup, load_block_textures);
        app.add_systems(
            Update,
            build_block_atlas.run_if(not(resource_exists::<BlockAtlas>)),
        );
    }
}

fn load_block_textures(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas: ResMut<UnbuiltBlockAtlas>,
    mut reg: ResMut<BlockTextureRegistry>,
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
            Ok(()) => println!(
                "Added {asset_path} to block atlas, registered as: {:?}, id->name={:?}",
                reg.name_to_id(&key),
                reg.id_to_name(id)
            ),
            Err(e) => eprintln!("Failed to add {asset_path} ({id}): {:?}", e),
        };
    }

    reg.freeze();
    atlas.mark_ready();

    commands.insert_resource(BlockTextureRegistryReady)
}

fn build_block_atlas(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<UnbuiltBlockAtlas>,
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
