use std::{collections::HashMap, marker::PhantomData};

use bevy::prelude::*;

#[derive(Debug)]
pub enum RegistryError<I> {
    DuplicateId(I),
    MissingId(I),
    Frozen
}

pub trait RegistryId: Copy + Eq + std::hash::Hash {
    fn from_index(index: usize) -> Self;
    fn to_index(self) -> usize;
}

#[derive(Debug)]
pub struct NameRegistry<I: RegistryId> {
    names_to_id: HashMap<String, I>,
    ids_to_name: Vec<String>,
    
    frozen: bool,
}

impl<I: RegistryId> Default for NameRegistry<I> {
    fn default() -> Self {
        Self {
            names_to_id: HashMap::new(),
            ids_to_name: Vec::new(),
            frozen: false
        }
    }
}

impl<I: RegistryId> NameRegistry<I> {
    pub fn register(&mut self, name: impl Into<String>) -> Result<I, RegistryError<I>> {
        if self.frozen {
            return Err(RegistryError::Frozen);
        }
        
        let name = name.into();
        
        if let Some(id) = self.names_to_id.get(&name) {
            return Ok(*id);
        }
        
        let id = I::from_index(self.ids_to_name.len());
        self.names_to_id.insert(name.clone(), id);
        self.ids_to_name.push(name);
        
        Ok(id)
    }
    
    pub fn freeze(&mut self) {
        self.frozen = true
    }
    
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }
    
    pub fn name_to_id(&self, name: &str) -> Option<I> {
        self.names_to_id.get(name).copied()
    }
    
    pub fn id_to_name(&self, id: I) -> Option<&str> {
        self.ids_to_name.get(id.to_index()).map(String::as_str)
    }
}

#[derive(Debug)]
pub struct LookupRegistry<I: RegistryId, V> {
    entries: Vec<Option<V>>,
    
    frozen: bool,
    _marker: PhantomData<I>
}

impl<I: RegistryId, V> Default for LookupRegistry<I, V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            frozen: false,
            
            _marker: PhantomData
        }
    }
}

impl<I: RegistryId, V> LookupRegistry<I, V> {
    pub fn insert_with_id(&mut self, id: I, value: V) -> Result<(), RegistryError<I>> {
        if self.frozen {
            return Err(RegistryError::Frozen)
        }
        
        let index = id.to_index();
        
        if self.entries.len() <= index {
            self.entries.resize_with(index + 1, || None);
        }
        
        if self.entries[index].is_some() {
            return Err(RegistryError::DuplicateId(id))
        }
        
        self.entries[index] = Some(value);
        Ok(())
    }
    
    pub fn insert(&mut self, value: V) -> Result<(), RegistryError<I>> {
        let index = self.entries.len();
        
        self.insert_with_id(I::from_index(index), value)
    }
    
    pub fn freeze(&mut self) {
        self.frozen = true
    }
    
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }
    
    pub fn get(&self, id: I) -> Option<&V> {
        self.entries.get(id.to_index())?.as_ref()
    }
}