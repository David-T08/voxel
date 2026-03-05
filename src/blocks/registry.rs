use bevy::prelude::*;
use std::collections::HashMap;

type BlockId = u16;

pub struct Block {
    name: String,
    
}

#[derive(Resource)]
pub struct BlockRegistry {
    blocks: Vec<Block>,
    names_to_ids: HashMap<String, BlockId>
    
}

