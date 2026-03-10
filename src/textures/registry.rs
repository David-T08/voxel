use bevy::prelude::*;

use crate::registry_base::NameRegistry;
pub use crate::registry_base::RegistryId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockTextureId(pub u16);

impl std::fmt::Display for BlockTextureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl RegistryId for BlockTextureId {
    fn from_index(index: usize) -> Self {
        Self(index as u16)
    }

    fn to_index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct BlockTextureRegistry(pub NameRegistry<BlockTextureId>);
