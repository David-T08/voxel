use std::sync::Arc;

use bevy::prelude::*;

pub use crate::registry_base::RegistryId;
use crate::registry_base::{LookupRegistry, NameRegistry};
use crate::textures::atlas::BlockAtlas;
use crate::textures::{BlockTextureId, BlockTextureRegistry, BlockTextures, BlockTexturesAsset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u16);

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum BlockRegistryError<I> {
    DuplicateId(I),
    MissingId(I),
    Frozen,
}

impl RegistryId for BlockId {
    fn from_index(index: usize) -> Self {
        Self(index as u16)
    }

    fn to_index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone)]
pub struct BlockDefinition {
    pub textures: BlockTextures,
    pub opaque: bool,
    pub emission: u8,
}

#[derive(Resource, Default)]
pub struct BlockRegistryInner {
    pub names: NameRegistry<BlockId>,
    pub definitions: LookupRegistry<BlockId, BlockDefinition>,

    frozen: bool,
}

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct BlockRegistry(pub Arc<BlockRegistryInner>);

impl BlockRegistryInner {
    pub fn freeze(&mut self) {
        self.definitions.freeze();
        self.names.freeze();
    }

    pub fn is_frozen(&self) -> bool {
        self.names.is_frozen() && self.definitions.is_frozen()
    }

    pub fn register_from_asset(
        &mut self,
        name: String,
        block_textures: BlockTexturesAsset,
        opaque: bool,
        tex_registry: &BlockTextureRegistry,
        atlas: &BlockAtlas,
    ) -> Option<BlockDefinition> {
        let id = self
            .names
            .register(name.clone())
            .expect("failed to register {name} because frozen name registry");
        let resolved = block_textures.resolve(tex_registry, atlas).unwrap();

        let def = BlockDefinition {
            textures: resolved,
            opaque,
            emission: if name == "core:stone2" { 15 } else { 0 },
        };

        let cloned = def.clone();
        self.definitions.insert_with_id(id, def);

        Some(cloned)
    }

    pub fn register_air(&mut self) {
        let id = self.names.register("core:air").unwrap();

        self.definitions
            .insert_with_id(
                id,
                BlockDefinition {
                    textures: BlockTextures::from_all([[0.0; 2]; 4]),
                    opaque: false,
                    emission: 0,
                },
            )
            .unwrap();
    }

    pub fn get_id(&self, name: impl Into<String>) -> Option<BlockId> {
        self.names.name_to_id(&name.into())
    }

    pub fn get_block(&self, id: BlockId) -> Option<&BlockDefinition> {
        self.definitions.get(id)
    }

    // pub fn get_block_by_name(&self, name: impl Into<String>) -> Option<&BlockDefinition> {

    // }

    pub fn is_opaque(&self, id: BlockId) -> bool {
        self.get_block(id).map(|block| block.opaque).unwrap_or(true)
    }
}
