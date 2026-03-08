use bevy::prelude::*;

pub use crate::registry_base::RegistryId;
use crate::registry_base::{LookupRegistry, NameRegistry};
use crate::textures::{BlockTextureId, BlockTextureRegistry, BlockTextures, BlockTexturesAsset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u16);

#[derive(Debug)]
pub enum BlockRegistryError<I> {
    DuplicateId(I),
    MissingId(I),
    Frozen
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
    pub textures: BlockTextures
}

#[derive(Resource, Default)]
pub struct BlockRegistry {
    pub names: NameRegistry<BlockId>,
    pub definitions: LookupRegistry<BlockId, BlockDefinition>,
    
    frozen: bool
}

impl BlockRegistry {
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
        tex_registry: &BlockTextureRegistry
    ) -> Option<BlockDefinition> {
        let id = self.names.register(name).expect("failed to register {name} because frozen name registry");
        let resolved = block_textures.resolve(tex_registry).unwrap();
        
        let def = BlockDefinition {
            textures: resolved
        };
        
        let cloned = def.clone();
        self.definitions.insert_with_id(id, def);
        
        Some(cloned)
    }
    
    pub fn register_air(&mut self) {
        let id = self.names.register("core:air").unwrap();
        
        self.definitions.insert_with_id(id, BlockDefinition { 
            textures: BlockTextures::from_all(BlockTextureId(0))
        }).unwrap();
    }
}