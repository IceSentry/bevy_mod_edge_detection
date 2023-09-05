use bevy::render::{
    render_resource::{
        encase::private::WriteInto, BindGroup, BindGroupDescriptor, BindGroupEntry,
        BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
        BindingType, BufferBinding, DynamicUniformBuffer, Sampler, ShaderStages, ShaderType,
        StorageBuffer, TextureSampleType, TextureView, UniformBuffer,
    },
    renderer::RenderDevice,
    texture::CachedTexture,
};

use self::bind_group_layout_types::storage_buffer;

pub trait RenderDeviceExt {
    fn create_bind_group_ext<const S: usize>(
        &self,
        label: &'static str,
        layout: &BindGroupLayout,
        entries: [BindGroupEntry; S],
    ) -> BindGroup;
    fn create_bind_group_layout_ext<const S: usize>(
        &self,
        label: &'static str,
        visibility: ShaderStages,
        entries: [BindingType; S],
    ) -> BindGroupLayout;
    fn create_bind_group_layout_ext2<const S: usize>(
        &self,
        label: &'static str,
        visibility: ShaderStages,
        entries: [BindGroupLayoutEntryWrapper; S],
    ) -> BindGroupLayout;
}

impl RenderDeviceExt for RenderDevice {
    #[inline]
    fn create_bind_group_ext<const S: usize>(
        &self,
        label: &'static str,
        layout: &BindGroupLayout,
        mut entries: [BindGroupEntry; S],
    ) -> BindGroup {
        let mut auto = false;
        for (index, entry) in entries.iter_mut().enumerate() {
            if entry.binding == u32::MAX {
                entry.binding = index as u32;
                auto = true;
            } else if auto {
                panic!("Cannot mix manual binding indices with automatic indices");
            }
        }
        self.create_bind_group(&BindGroupDescriptor {
            label: if label.is_empty() { None } else { Some(label) },
            layout,
            entries: &entries,
        })
    }

    fn create_bind_group_layout_ext<const S: usize>(
        &self,
        label: &'static str,
        visibility: ShaderStages,
        entries: [BindingType; S],
    ) -> BindGroupLayout {
        let entries = entries
            .iter()
            .enumerate()
            .map(|(i, ty)| BindGroupLayoutEntry {
                binding: i as u32,
                visibility,
                ty: *ty,
                count: None,
            })
            .collect::<Vec<_>>();
        self.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: if label.is_empty() { None } else { Some(label) },
            entries: &entries,
        })
    }

    fn create_bind_group_layout_ext2<const S: usize>(
        &self,
        label: &'static str,
        default_visibility: ShaderStages,
        entries: [BindGroupLayoutEntryWrapper; S],
    ) -> BindGroupLayout {
        let entries = entries
            .iter()
            .enumerate()
            .map(|(i, entry)| BindGroupLayoutEntry {
                binding: i as u32,
                visibility: entry.override_vis.unwrap_or(default_visibility),
                ty: entry.raw.ty,
                count: None,
            })
            .collect::<Vec<_>>();
        self.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: if label.is_empty() { None } else { Some(label) },
            entries: &entries,
        })
    }
}

pub trait BindingResouceExt {
    fn bind_index(&self, binding_index: u32) -> BindGroupEntry;
    fn bind(&self) -> BindGroupEntry;
}
impl<T: ShaderType + WriteInto> BindingResouceExt for UniformBuffer<T> {
    #[inline]
    fn bind_index(&self, binding_index: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding: binding_index,
            resource: BindingResource::Buffer(
                self.buffer()
                    .expect("Failed to get buffer")
                    .as_entire_buffer_binding(),
            ),
        }
    }
    fn bind(&self) -> BindGroupEntry {
        self.bind_index(u32::MAX)
    }
}
impl<T: ShaderType + WriteInto> BindingResouceExt for StorageBuffer<T> {
    #[inline]
    fn bind_index(&self, binding_index: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding: binding_index,
            resource: BindingResource::Buffer(
                self.buffer()
                    .expect("Failed to get buffer")
                    .as_entire_buffer_binding(),
            ),
        }
    }
    fn bind(&self) -> BindGroupEntry {
        self.bind_index(u32::MAX)
    }
}
impl BindingResouceExt for TextureView {
    #[inline]
    fn bind_index(&self, binding_index: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding: binding_index,
            resource: BindingResource::TextureView(self),
        }
    }

    #[inline]
    fn bind(&self) -> BindGroupEntry {
        self.bind_index(u32::MAX)
    }
}
impl<T: ShaderType + WriteInto> BindingResouceExt for DynamicUniformBuffer<T> {
    #[inline]
    fn bind_index(&self, binding_index: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding: binding_index,
            resource: BindingResource::Buffer(BufferBinding {
                buffer: self.buffer().expect("Failed to get buffer"),
                offset: 0,
                size: Some(T::min_size()),
            }),
        }
    }

    #[inline]
    fn bind(&self) -> BindGroupEntry {
        self.bind_index(u32::MAX)
    }
}
impl BindingResouceExt for Sampler {
    #[inline]
    fn bind_index(&self, binding_index: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding: binding_index,
            resource: BindingResource::Sampler(self),
        }
    }

    #[inline]
    fn bind(&self) -> BindGroupEntry {
        self.bind_index(u32::MAX)
    }
}

impl BindingResouceExt for CachedTexture {
    #[inline]
    fn bind_index(&self, binding_index: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding: binding_index,
            resource: BindingResource::TextureView(&self.default_view),
        }
    }

    #[inline]
    fn bind(&self) -> BindGroupEntry {
        self.bind_index(u32::MAX)
    }
}

impl BindingResouceExt for Option<CachedTexture> {
    #[inline]
    fn bind_index(&self, binding_index: u32) -> BindGroupEntry {
        BindGroupEntry {
            binding: binding_index,
            resource: BindingResource::TextureView(&self.as_ref().unwrap().default_view),
        }
    }

    #[inline]
    fn bind(&self) -> BindGroupEntry {
        self.bind_index(u32::MAX)
    }
}

struct BindGroupLayoutEntryWrapper {
    override_vis: Option<ShaderStages>,
    ty: BindingType,
}

impl BindGroupLayoutEntryWrapper {
    fn visibility(mut self, override_vis: ShaderStages) -> Self {
        self.override_vis = Some(override_vis);
        self
    }

    fn from_ty(ty: BindingType) -> Self {
        Self {
            override_vis: None,
            ty,
        }
    }
}

fn temp(render_device: RenderDevice) {
    use bind_group_layout_types2::*;
    render_device.create_bind_group_layout_ext2(
        "label",
        ShaderStages::FRAGMENT,
        [
            storage_buffer(false, None),
            texture_2d(TextureSampleType::Float { filterable: true })
                .visibility(ShaderStages::VERTEX_FRAGMENT),
            texture_2d_multisampled(TextureSampleType::Float { filterable: true }),
        ],
    );
}
pub mod bind_group_layout_types2 {
    use std::num::NonZeroU64;

    use bevy::render::render_resource::{
        BindingType, BufferBindingType, TextureSampleType, TextureViewDimension,
    };

    use super::BindGroupLayoutEntryWrapper;

    #[allow(unused)]
    pub fn storage_buffer(
        has_dynamic_offset: bool,
        min_binding_size: Option<NonZeroU64>,
    ) -> BindGroupLayoutEntryWrapper {
        BindGroupLayoutEntryWrapper {
            override_vis: None,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset,
                min_binding_size,
            },
        }
    }

    #[allow(unused)]
    pub fn texture_2d(sample_type: TextureSampleType) -> BindGroupLayoutEntryWrapper {
        BindGroupLayoutEntryWrapper::from_ty(BindingType::Texture {
            sample_type,
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        })
    }

    #[allow(unused)]
    pub fn texture_2d_multisampled(sample_type: TextureSampleType) -> BindGroupLayoutEntryWrapper {
        BindGroupLayoutEntryWrapper::from_ty(BindingType::Texture {
            sample_type,
            view_dimension: TextureViewDimension::D2,
            multisampled: true,
        })
    }
}

pub mod bind_group_layout_types {
    use std::num::NonZeroU64;

    use bevy::render::render_resource::{
        BindingType, BufferBindingType, SamplerBindingType, TextureSampleType, TextureViewDimension,
    };

    #[allow(unused)]
    pub fn storage_buffer(
        has_dynamic_offset: bool,
        min_binding_size: Option<NonZeroU64>,
    ) -> BindingType {
        BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: false },
            has_dynamic_offset,
            min_binding_size,
        }
    }

    #[allow(unused)]
    pub fn storage_buffer_read_only(
        has_dynamic_offset: bool,
        min_binding_size: Option<NonZeroU64>,
    ) -> BindingType {
        BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: true },
            has_dynamic_offset,
            min_binding_size,
        }
    }

    #[allow(unused)]
    pub fn uniform_buffer(
        has_dynamic_offset: bool,
        min_binding_size: Option<NonZeroU64>,
    ) -> BindingType {
        BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset,
            min_binding_size,
        }
    }

    #[allow(unused)]
    pub fn texture_2d(sample_type: TextureSampleType) -> BindingType {
        BindingType::Texture {
            sample_type,
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        }
    }

    #[allow(unused)]
    pub fn texture_2d_multisampled(sample_type: TextureSampleType) -> BindingType {
        BindingType::Texture {
            sample_type,
            view_dimension: TextureViewDimension::D2,
            multisampled: true,
        }
    }

    #[allow(unused)]
    pub fn texture_depth_2d() -> BindingType {
        BindingType::Texture {
            sample_type: TextureSampleType::Depth,
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        }
    }

    #[allow(unused)]
    pub fn texture_depth_2d_multisampled() -> BindingType {
        BindingType::Texture {
            sample_type: TextureSampleType::Depth,
            view_dimension: TextureViewDimension::D2,
            multisampled: true,
        }
    }

    #[allow(unused)]
    pub fn sampler(sampler_binding_type: SamplerBindingType) -> BindingType {
        BindingType::Sampler(sampler_binding_type)
    }
}
