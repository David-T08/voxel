use std::collections::HashMap;

use bevy::prelude::*;

pub trait RegistryId: Copy + Eq + std::hash::Hash {
    fn from_index(index: usize) -> Self;
    fn to_index(self) -> usize;
}

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

#[derive(Debug)]
pub struct TextureRegistry<I: RegistryId> {
    names_to_id: HashMap<String, I>,
    ids_to_name: Vec<String>,
    
    frozen: bool,
}

impl<I: RegistryId> Default for TextureRegistry<I> {
    fn default() -> Self {
        Self {
            names_to_id: HashMap::new(),
            ids_to_name: Vec::new(),
            frozen: false
        }
    }
}

impl<I> TextureRegistry<I> 
where
    I: RegistryId + Copy
{
    pub fn register(&mut self, name: impl Into<String>) -> Option<I> {
        if self.frozen {
            return None;
        }
        
        let name = name.into();
        
        if let Some(id) = self.names_to_id.get(&name) {
            return Some(*id);
        }
        
        let id = I::from_index(self.ids_to_name.len());
        self.names_to_id.insert(name.clone(), id);
        self.ids_to_name.push(name);
        
        Some(id)
    }
    
    pub fn freeze(&mut self) {
        self.frozen = true
    }
    
    pub fn name_to_id(&self, name: &str) -> Option<I> {
        self.names_to_id.get(name).copied()
    }
    
    pub fn id_to_name(&self, id: I) -> Option<&str> {
        self.ids_to_name.get(id.to_index()).map(String::as_str)
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct BlockTextureRegistry(pub TextureRegistry<BlockTextureId>);