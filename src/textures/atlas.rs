use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use std::marker::PhantomData;
use std::u16;

use crate::textures::registry::{BlockTextureId, RegistryId};

pub const MINIMUM_TEXTURE_SIZE: u32 = 2;
pub const MAXIMUM_TEXTURE_SIZE: u32 = 64;
pub const ATLAS_PADDING: u32 = 0;

#[derive(Debug)]
pub enum AtlasError<I: RegistryId> {
    DuplicateId(I),
    MissingId(I),
    NonSquareAspectRatio(I),
    NotPowerOfTwo {
        id: I,
        got: u32,
    },
    TextureSizeMismatched {
        id: I,
        got: u32,
        expected: u32,
    },
    TextureSizeOutOfBounds {
        id: I,
        maximum: u32,
        minimum: u32,
        got: u32,
    },
    NotReady,
}

#[derive(Debug)]
pub struct AtlasReadyStatus {
    pub ready: bool,
    pub pending: u16,
    pub loaded: u16,
    pub total: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    pub pixel_pos: UVec2,
    pub pixel_size: UVec2,
    pub uv_min: Vec2,
    pub uv_max: Vec2,
}

#[derive(Debug)]
pub struct UnbuiltAtlas<I: RegistryId> {
    entries: Vec<Option<Handle<Image>>>,
    // Marker to signify we've put in all the textures we want to
    ready: bool,
    _marker: PhantomData<I>,
}

impl<I> Default for UnbuiltAtlas<I>
where
    I: RegistryId,
{
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            ready: false,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct BuiltAtlas<I: RegistryId> {
    pub atlas: Handle<Image>,
    pub entries: Vec<AtlasRegion>,
    pub texture_size: u32,
    pub atlas_size: u32,
    _marker: PhantomData<I>,
}

fn copy_into_image(dst: &mut Image, src: &Image, dst_x: u32, dst_y: u32) {
    let dst_w = dst.width();
    let src_w = src.width();
    let src_h = src.height();

    let dst_data = dst.data.as_mut().expect("Destination image has no data");
    let src_data = src.data.as_ref().expect("Source image has no data");

    for y in 0..src_h {
        for x in 0..src_w {
            let src_i = ((y * src_w + x) * 4) as usize;
            let dst_i = ((((dst_y + y) * dst_w) + (dst_x + x)) * 4) as usize;

            dst_data[dst_i..dst_i + 4].copy_from_slice(&src_data[src_i..src_i + 4]);
        }
    }
}

impl<I> UnbuiltAtlas<I>
where
    I: RegistryId + Clone,
{
    pub fn insert(&mut self, id: I, image: Handle<Image>) -> Result<(), AtlasError<I>> {
        let index = id.to_index();

        if self.entries.len() <= index {
            self.entries.resize(index + 1, None);
        }

        if self.entries[index].is_some() {
            return Err(AtlasError::DuplicateId(id));
        }

        self.entries[index] = Some(image);
        Ok(())
    }

    pub fn mark_ready(&mut self) {
        self.ready = true;
    }

    pub fn ready_status(&self, assets: &AssetServer, images: &Assets<Image>) -> AtlasReadyStatus {
        if !self.ready {
            return AtlasReadyStatus {
                ready: false,
                pending: 0,
                loaded: u16::MAX,
                total: 0,
            };
        }

        let mut pending: u16 = 0;
        let mut total: u16 = 0;

        for tex in &self.entries {
            total += 1;

            match tex {
                Some(handle) => {
                    if !assets.is_loaded_with_dependencies(handle) || images.get(handle).is_none() {
                        pending += 1;
                    }
                }
                None => {
                    pending += 1;
                }
            }
        }

        AtlasReadyStatus {
            ready: total > 0 && pending == 0,
            loaded: total - pending,
            pending,
            total,
        }
    }

    // Returns the texture_size if ok
    pub fn verify(&self, images: &Assets<Image>) -> Result<u32, Vec<AtlasError<I>>> {
        let mut errs: Vec<AtlasError<I>> = Vec::new();
        let mut tex_size: u32 = 0;

        for (id, tex) in self.entries.iter().enumerate() {
            let id = I::from_index(id);

            let tex = match tex {
                Some(handle) => handle,
                None => {
                    errs.push(AtlasError::MissingId(id));
                    continue;
                }
            };

            let image = images.get(tex).unwrap();

            let w = image.width();
            let h = image.height();

            let mut had_error = false;
            if w != h {
                errs.push(AtlasError::NonSquareAspectRatio(id));
            }

            if !w.is_power_of_two() {
                errs.push(AtlasError::NotPowerOfTwo { id: id, got: w });
                had_error = true;
            } else if !h.is_power_of_two() {
                errs.push(AtlasError::NotPowerOfTwo { id: id, got: h });
                had_error = true;
            }

            if w > MAXIMUM_TEXTURE_SIZE
                || h > MAXIMUM_TEXTURE_SIZE
                || w < MINIMUM_TEXTURE_SIZE
                || h < MINIMUM_TEXTURE_SIZE
            {
                errs.push(AtlasError::TextureSizeOutOfBounds {
                    id: id,
                    got: w,
                    minimum: MINIMUM_TEXTURE_SIZE,
                    maximum: MAXIMUM_TEXTURE_SIZE,
                });
            }

            if tex_size == 0 && !had_error {
                tex_size = w;
            } else if w != tex_size || h != tex_size {
                errs.push(AtlasError::TextureSizeMismatched {
                    id: id,
                    got: w,
                    expected: tex_size,
                });
            }
        }

        if !errs.is_empty() {
            return Err(errs);
        }

        Ok(tex_size)
    }

    pub fn build(
        self,
        assets: &AssetServer,
        images: &mut Assets<Image>,
    ) -> Result<BuiltAtlas<I>, (Vec<AtlasError<I>>, UnbuiltAtlas<I>)> {
        if !self.ready_status(assets, images).ready {
            return Err((vec![AtlasError::NotReady], self));
        }

        let tex_size = match self.verify(images) {
            Ok(s) => s,
            Err(e) => return Err((e, self)),
        };

        let count = self.entries.len() as u32;
        let cells_per_axis = (count as f32).sqrt().ceil() as u32;
        let atlas_size = (cells_per_axis * (tex_size + ATLAS_PADDING)).next_power_of_two();

        let extent = Extent3d {
            width: atlas_size,
            height: atlas_size,
            depth_or_array_layers: 1,
        };

        let mut atlas = Image::new_fill(
            extent,
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );

        atlas.sampler = ImageSampler::nearest();

        let mut layout: Vec<AtlasRegion> = Vec::with_capacity(self.entries.len());
        for (i, tex) in self.entries.iter().enumerate() {
            let tex = tex.as_ref().unwrap();
            let src = images.get(tex).unwrap();

            let i = i as u32;
            let cell_x = i % cells_per_axis;
            let cell_y = i / cells_per_axis;

            let dst_x = cell_x * (tex_size + ATLAS_PADDING);
            let dst_y = cell_y * (tex_size + ATLAS_PADDING);

            copy_into_image(&mut atlas, src, dst_x, dst_y);

            let uv_min = Vec2::new(
                dst_x as f32 / atlas_size as f32,
                dst_y as f32 / atlas_size as f32,
            );

            let uv_max = Vec2::new(
                (dst_x + tex_size) as f32 / atlas_size as f32,
                (dst_y + tex_size) as f32 / atlas_size as f32,
            );

            layout.push(AtlasRegion {
                pixel_pos: UVec2::new(dst_x, dst_y),
                pixel_size: UVec2::new(tex_size, tex_size),
                uv_min,
                uv_max,
            });
        }

        let atlas_handle = images.add(atlas);

        Ok(BuiltAtlas {
            atlas: atlas_handle,
            entries: layout,
            texture_size: tex_size,
            atlas_size,
            _marker: PhantomData,
        })
    }
}

impl<I> BuiltAtlas<I>
where
    I: Clone + RegistryId,
{
    pub fn get(&self, id: I) -> Result<&AtlasRegion, AtlasError<I>> {
        self.entries
            .get(id.to_index())
            .ok_or(AtlasError::MissingId(id))
    }

    pub fn face_uvs(&self, id: I) -> Result<[[f32; 2]; 4], AtlasError<I>> {
        let region = self.get(id)?;

        Ok([
            [region.uv_min.x, region.uv_max.y],
            [region.uv_max.x, region.uv_max.y],
            [region.uv_max.x, region.uv_min.y],
            [region.uv_min.x, region.uv_min.y],
        ])
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct UnbuiltBlockAtlas(pub UnbuiltAtlas<BlockTextureId>);

#[derive(Resource, Deref, DerefMut)]
pub struct BlockAtlas(pub BuiltAtlas<BlockTextureId>);
