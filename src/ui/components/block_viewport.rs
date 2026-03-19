// TODO: Make it dynamic eventually

use bevy::{asset::RenderAssetUsages, camera::{RenderTarget, ScalingMode}, mesh::Indices, prelude::*, render::render_resource::TextureFormat};
use crate::{blocks::{BlockId, BlockRegistry}, registry_base::{LookupRegistry, RegistryId}, textures::atlas::BlockAtlas};


const GRID_SIZE: usize = 10;
pub const MAX_TEXTURE_CACHE: usize = GRID_SIZE * GRID_SIZE;
pub const ICON_SIZE: usize = 64;
pub const ATLAS_SIZE: usize = ICON_SIZE * GRID_SIZE;
pub const GRID_SPACING: usize = 2;

#[derive(Component)]
pub struct BlockBakerCamera;

#[derive(Resource)]
pub enum BlockBakeState {
    WaitingForRender,
    Idle
}

#[derive(Resource)]
pub struct BlockIconCache {
    pub icons: LookupRegistry<BlockId, [f32; 4]>,
    pub atlas: Handle<Image>,
    
    needs_rebake: bool,
    queued_blocks: Vec<BlockId>,
    
    meshes: Vec<Entity>,
}

#[derive(Resource)]
pub struct BlockIconCacheReady;

pub fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>
) {
    let target = Image::new_target_texture(
        ATLAS_SIZE as u32, 
        ATLAS_SIZE as u32, 
        TextureFormat::Rgba8UnormSrgb, 
        Some(TextureFormat::Rgba8UnormSrgb)
    );
    
    let handle = images.add(target);
    let cube_spacing = (GRID_SPACING + 1) as f32;
    let extent = (GRID_SIZE + (GRID_SIZE - 1) * GRID_SPACING) as f32;
    
    let center = Vec3::new(
        (GRID_SIZE as f32  - 1.0) * cube_spacing * 0.5,
        0.5,
        (GRID_SIZE as f32  - 1.0) * cube_spacing * 0.5,
    );
    
    commands.spawn((
        BlockBakerCamera,
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed { 
                width: extent,
                height: extent
            },
            near: 0.0,
            far: 1000.0,
            ..OrthographicProjection::default_3d()
        }),
        Camera {
            order: -1,
            clear_color: Color::WHITE.into(),
            ..default()
        },
        Transform::from_xyz(center.x, 100.0, center.z)
            .looking_at(center, Vec3::NEG_Z),
        RenderTarget::Image(handle.clone().into())
    ));
    
    commands.insert_resource(BlockIconCache {
        icons: LookupRegistry::default(),
        atlas: handle,
        
        needs_rebake: false,
        queued_blocks: Vec::new(),
        meshes: Vec::new()
    });
    
    commands.insert_resource(BlockBakeState::Idle);
}

pub fn bake_images(
    mut commands: Commands,
    mut cache: ResMut<BlockIconCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<BlockBakeState>,
    atlas: Res<BlockAtlas>,
    blocks: Res<BlockRegistry>
) {
    if !cache.needs_rebake || !matches!(*state, BlockBakeState::Idle) {
        return
    }
    
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(atlas.atlas.clone()),
        ..default()
    });
    
    cache.icons = LookupRegistry::default();
    
    let queued: Vec<_> = cache.queued_blocks.drain(..).collect();
    for (i, queued) in queued.into_iter().enumerate() {
        let cube = meshes.add(build_cube(&blocks, queued));
        
        let x = (i % GRID_SIZE) as f32;
        let y = (i / GRID_SIZE) as f32; 
        
        let min_x = x * ICON_SIZE as f32;
        let min_y = y * ICON_SIZE as f32;
        let max_x = min_x + ICON_SIZE as f32;
        let max_y = min_y + ICON_SIZE as f32;
        
        cache.icons.insert_with_id(queued, [min_x, min_y, max_x, max_y]);
        
        let entity = commands
            .spawn((
                Mesh3d(cube),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(Vec3::new(
                    x * (1.0 + GRID_SPACING as f32),
                    0.0,
                    y * (1.0 + GRID_SPACING as f32)
                )),
                ))
                .id();
        
        cache.meshes.push(entity)
    }
    
    *state = BlockBakeState::WaitingForRender
}

pub fn finalize_bake(
    mut commands: Commands,
    mut cache: ResMut<BlockIconCache>,
    mut state: ResMut<BlockBakeState>,
) {
    if !cache.needs_rebake {
        return;
    }
    
    if matches!(*state, BlockBakeState::WaitingForRender) {
        *state = BlockBakeState::Idle;
        return
    }
    
    for entity in cache.meshes.drain(..) {
        commands.entity(entity).despawn();
    }
    
    cache.needs_rebake = false;
}

impl BlockIconCache {
    pub fn get(&self, id: BlockId) -> Option<[f32; 4]> {
        self.icons.get(id).cloned()
    }
    
    pub fn populate(&mut self, blocks: &BlockRegistry) {
        if self.needs_rebake {
            return;
        }
        
        self.queued_blocks.clear();
        
        let max = blocks.definitions.entries.len().min(MAX_TEXTURE_CACHE + 1);
        for i in 1..max {
            self.queued_blocks.push(BlockId::from_index(i));
        }
    }
    
    pub fn mark_rebake(&mut self) {
        self.needs_rebake = true;
    }
}

#[rustfmt::skip]
fn build_cube(blocks: &BlockRegistry, id: BlockId) -> Mesh {
    const POSITIONS: [[f32; 3]; 24] = [
        // Top face
        [-0.5,  0.5,  0.5], // BL
        [ 0.5,  0.5,  0.5], // BR
        [ 0.5,  0.5, -0.5], // TR
        [-0.5,  0.5, -0.5], // TL  

        // Bottom face
        [-0.5, -0.5, -0.5], // BL
        [ 0.5, -0.5, -0.5], // BR
        [ 0.5, -0.5,  0.5], // TR
        [-0.5, -0.5,  0.5], // TL
            
        // Front face
        [-0.5, -0.5,  0.5], // BL
        [ 0.5, -0.5,  0.5], // BR
        [ 0.5,  0.5,  0.5], // TR
        [-0.5,  0.5,  0.5], // TL

        // Back face
        [ 0.5, -0.5, -0.5], // BL
        [-0.5, -0.5, -0.5], // BR
        [-0.5,  0.5, -0.5], // TR
        [ 0.5,  0.5, -0.5], // TL

        // Left face
        [-0.5, -0.5, -0.5],
        [-0.5, -0.5,  0.5],
        [-0.5,  0.5,  0.5],
        [-0.5,  0.5, -0.5],

        // Right face
        [0.5, -0.5,  0.5],
        [0.5, -0.5, -0.5],
        [0.5,  0.5, -0.5],
        [0.5,  0.5,  0.5],
    ];
    
    const NORMALS: [[f32; 3]; 24] = [
        [ 0.0,  1.0,  0.0],
        [ 0.0,  1.0,  0.0],
        [ 0.0,  1.0,  0.0],
        [ 0.0,  1.0,  0.0],
        
        [ 0.0, -1.0,  0.0],
        [ 0.0, -1.0,  0.0],
        [ 0.0, -1.0,  0.0],
        [ 0.0, -1.0,  0.0],
        
        [ 0.0,  0.0,  1.0],
        [ 0.0,  0.0,  1.0],
        [ 0.0,  0.0,  1.0],
        [ 0.0,  0.0,  1.0],
        
        [ 0.0,  0.0, -1.0],
        [ 0.0,  0.0, -1.0],
        [ 0.0,  0.0, -1.0],
        [ 0.0,  0.0, -1.0],
        
        [-1.0,  0.0,  0.0],
        [-1.0,  0.0,  0.0],
        [-1.0,  0.0,  0.0],
        [-1.0,  0.0,  0.0],
        
        [ 1.0,  0.0,  0.0],
        [ 1.0,  0.0,  0.0],
        [ 1.0,  0.0,  0.0],
        [ 1.0,  0.0,  0.0],
        
    ];
    
    let uvs = blocks.get_block(id).expect("Expected block to exist").textures.get_uvs_flat();
    let mut indices: [u32; 36] = [0; 36];
    
    for face in 0..6 {
        let vi = (face * 4) as u32;
        let ii = face * 6;
    
        indices[ii..ii + 6].copy_from_slice(&[
            vi, vi + 1, vi + 2,
            vi, vi + 2, vi + 3,
        ]);
    }
    
    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, POSITIONS.to_vec())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, NORMALS.to_vec())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs.to_vec())
        .with_inserted_indices(Indices::U32(indices.to_vec()))
}