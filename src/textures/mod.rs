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
    pub fn resolve(self, reg: &BlockTextureRegistry, atlas: &BlockAtlas) -> Result<BlockTextures, String> {
        fn get_tex(reg: &BlockTextureRegistry, name: &str, atlas: &BlockAtlas) -> Result<[[f32; 2]; 4], String> {
            reg.name_to_id(name)
                .ok_or_else(|| format!("unknown block texture: {name}"))
                .map(|id| atlas.face_uvs(id).unwrap())
        }

        if let Some(all) = self.all {
            return Ok(BlockTextures::from_all(get_tex(reg, &all, atlas)?));
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
                get_tex(reg, top, atlas)?,
                get_tex(reg, bottom, atlas)?,
                get_tex(reg, &side, atlas)?,
            ));
        }

        Ok(BlockTextures::new(
            get_tex(
                reg,
                self.top
                    .as_deref()
                    .ok_or_else(|| "missing top texture".to_string())?,
                atlas
            )?,
            get_tex(
                reg,
                self.bottom
                    .as_deref()
                    .ok_or_else(|| "missing bottom texture".to_string())?,
                atlas
            )?,
            get_tex(
                reg,
                self.front
                    .as_deref()
                    .ok_or_else(|| "missing front texture".to_string())?,
                atlas
            )?,
            get_tex(
                reg,
                self.back
                    .as_deref()
                    .ok_or_else(|| "missing back texture".to_string())?,
                atlas
            )?,
            get_tex(
                reg,
                self.left
                    .as_deref()
                    .ok_or_else(|| "missing left texture".to_string())?,
                atlas
            )?,
            get_tex(
                reg,
                self.right
                    .as_deref()
                    .ok_or_else(|| "missing right texture".to_string())?,
                atlas
            )?,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct BlockTextures {
    pub top: [[f32; 2]; 4],
    pub bottom: [[f32; 2]; 4],

    pub front: [[f32; 2]; 4],
    pub back: [[f32; 2]; 4],

    pub left: [[f32; 2]; 4],
    pub right: [[f32; 2]; 4],
}

impl BlockTextures {
    pub fn new(
        top: [[f32; 2]; 4],
        bottom: [[f32; 2]; 4],
        front: [[f32; 2]; 4],
        back: [[f32; 2]; 4],
        left: [[f32; 2]; 4],
        right: [[f32; 2]; 4],
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
        top: [[f32; 2]; 4],
        bottom: [[f32; 2]; 4],
        side: [[f32; 2]; 4],
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

    pub fn from_all(all: [[f32; 2]; 4]) -> Self {
        Self {
            top: all,
            bottom: all,

            front: all,
            back: all,

            left: all,
            right: all,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = [[f32; 2]; 4]> {
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

    pub fn get_uvs(&self) -> [[[f32; 2]; 4]; 6] {
        [
            self.top,
            self.bottom,
            self.front,
            self.back,
            self.left,
            self.right,
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
