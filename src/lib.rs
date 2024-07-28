use bevy::{
    asset::load_internal_asset,
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{RenderGraphApp, ViewNodeRunner},
        render_resource::{
            binding_types::{
                sampler, texture_2d, texture_depth_2d, uniform_buffer, uniform_buffer_sized,
            },
            BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, FragmentState, MultisampleState, PipelineCache, PrimitiveState,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            ShaderType, TextureFormat, TextureSampleType, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::ViewUniform,
        Extract, Render, RenderApp, RenderSet,
    },
};
use node::EdgeDetectionNode;

use crate::node::EdgeDetetctionNodeLabel;

mod node;

pub const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(410592619790336);

pub struct EdgeDetectionPlugin;
impl Plugin for EdgeDetectionPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, SHADER_HANDLE, "edge_detection.wgsl", Shader::from_wgsl);
        // app.add_systems(Update, print_projection);

        app.add_plugins(ExtractComponentPlugin::<EdgeDetectionCamera>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(ExtractSchedule, extract_config)
            .add_systems(Render, prepare_config_buffer.in_set(RenderSet::Prepare));

        render_app
            .add_render_graph_node::<ViewNodeRunner<EdgeDetectionNode>>(
                Core3d,
                EdgeDetetctionNodeLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::EndMainPass,
                    EdgeDetetctionNodeLabel,
                    Node3d::Tonemapping,
                ),
            );
    }
    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<EdgeDetectionPipeline>()
            .init_resource::<ConfigBuffer>();
    }
}
#[derive(Component, Clone, Copy, ExtractComponent)]
pub struct EdgeDetectionCamera;

#[derive(Resource, ShaderType, Clone, Copy)]
pub struct EdgeDetectionConfig {
    pub depth_threshold: f32,
    pub normal_threshold: f32,
    pub color_threshold: f32,
    pub edge_color: Vec4,
    pub debug: u32,
    pub enabled: u32,
}

impl Default for EdgeDetectionConfig {
    fn default() -> Self {
        Self {
            depth_threshold: 0.2,
            normal_threshold: 0.05,
            color_threshold: 1.0,
            edge_color: Color::BLACK.to_linear().to_vec4(),
            debug: 0,
            enabled: 1,
        }
    }
}

#[derive(Resource)]
struct ConfigBuffer {
    buffer: UniformBuffer<EdgeDetectionConfig>,
}

impl FromWorld for ConfigBuffer {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let config = EdgeDetectionConfig::default();
        let mut buffer = UniformBuffer::default();
        buffer.set(config);
        buffer.write_buffer(render_device, render_queue);

        ConfigBuffer { buffer }
    }
}

fn extract_config(mut commands: Commands, config: Extract<Res<EdgeDetectionConfig>>) {
    commands.insert_resource(**config);
}

fn prepare_config_buffer(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut config_buffer: ResMut<ConfigBuffer>,
    config: Res<EdgeDetectionConfig>,
) {
    let buffer = config_buffer.buffer.get_mut();
    *buffer = *config;
    config_buffer
        .buffer
        .write_buffer(&render_device, &render_queue);
}

#[derive(Resource)]
struct EdgeDetectionPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for EdgeDetectionPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "edge_detection_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // screen_texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    // depth prepass
                    texture_depth_2d(),
                    // normal prepass
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // view
                    uniform_buffer::<ViewUniform>(true),
                    // config
                    uniform_buffer_sized(false, None),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("edge_detection_pipeline".into()),
                    layout: vec![layout.clone()],
                    // This will setup a fullscreen triangle for the vertex state
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader: SHADER_HANDLE,
                        shader_defs: vec!["VIEW_PROJECTION_PERSPECTIVE".into()], // TODO detect projection
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::bevy_default(),
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}
