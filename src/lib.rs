use bevy::{
    core_pipeline::{
        core_3d, fullscreen_vertex_shader::fullscreen_shader_vertex_state,
        prepass::ViewPrepassTextures,
    },
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotInfo, SlotType},
        render_resource::{
            BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
            BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
            Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderDefVal, ShaderStages, ShaderType, TextureFormat,
            TextureSampleType, TextureViewDimension, UniformBuffer,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget, ViewUniforms},
        Extract, RenderApp, RenderSet,
    },
};

pub struct EdgeDetectionPlugin;
impl Plugin for EdgeDetectionPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<EdgeDetectionPipeline>()
            .init_resource::<ConfigBuffer>()
            .add_system(extract_config.in_schedule(ExtractSchedule))
            .add_system(prepare_config_buffer.in_set(RenderSet::Prepare));

        let node = EdgeDetectionNode::new(&mut render_app.world);

        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        let core_3d_graph = graph.get_sub_graph_mut(core_3d::graph::NAME).unwrap();
        core_3d_graph.add_node(EdgeDetectionNode::NAME, node);

        core_3d_graph.add_slot_edge(
            core_3d_graph.input_node().id,
            core_3d::graph::input::VIEW_ENTITY,
            EdgeDetectionNode::NAME,
            EdgeDetectionNode::IN_VIEW,
        );

        core_3d_graph.add_node_edge(core_3d::graph::node::MAIN_PASS, EdgeDetectionNode::NAME);
        core_3d_graph.add_node_edge(EdgeDetectionNode::NAME, core_3d::graph::node::TONEMAPPING);
    }
}

struct EdgeDetectionNode {
    query: QueryState<(&'static ViewTarget, &'static ViewPrepassTextures), With<ExtractedView>>,
}

impl EdgeDetectionNode {
    pub const IN_VIEW: &str = "view";
    pub const NAME: &str = "edge_detection";

    fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl Node for EdgeDetectionNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(EdgeDetectionNode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph_context: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph_context.get_input_entity(EdgeDetectionNode::IN_VIEW)?;

        let Ok((view_target, prepass_textures)) = self.query.get_manual(world, view_entity) else {
            return Ok(());
        };

        let post_process_pipeline = world.resource::<EdgeDetectionPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id) else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();
        let source = post_process.source;
        let destination = post_process.destination;

        let view_uniforms = world.resource::<ViewUniforms>();

        let Some(view_binding) = view_uniforms.uniforms.binding() else {
            return Ok(());
        };

        let config_buffer = world.resource::<ConfigBuffer>();

        let bind_group_descriptor = BindGroupDescriptor {
            label: Some("edge_detection_bind_group"),
            layout: &post_process_pipeline.layout,
            entries: &[
                // screen texture
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(source),
                },
                // sampler
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&post_process_pipeline.sampler),
                },
                // depth
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(
                        &prepass_textures.depth.as_ref().unwrap().default_view,
                    ),
                },
                // normal
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(
                        &prepass_textures.normal.as_ref().unwrap().default_view,
                    ),
                },
                // view
                BindGroupEntry {
                    binding: 4,
                    resource: view_binding.clone(),
                },
                // config
                BindGroupEntry {
                    binding: 5,
                    resource: config_buffer.buffer.binding().unwrap().clone(),
                },
            ],
        };

        let bind_group = render_context
            .render_device()
            .create_bind_group(&bind_group_descriptor);

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("edge_detection_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource, ShaderType, Clone, Copy)]
pub struct EdgeDetectionConfig {
    pub depth_threshold: f32,
    pub normal_threshold: f32,
    pub color_threshold: f32,
    pub edge_color: Color,
    pub debug: f32,
    pub enabled: f32,
}

impl Default for EdgeDetectionConfig {
    fn default() -> Self {
        Self {
            depth_threshold: 0.2,
            normal_threshold: 0.05,
            color_threshold: 1.0,
            edge_color: Color::BLACK,
            debug: 0.0,
            enabled: 1.0,
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

        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("edge_detection_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Depth
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Depth,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Normals
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // View
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Config
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world.resource::<AssetServer>().load("edge_detection.wgsl");

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("edge_detection_pipeline".into()),
                    layout: vec![layout.clone()],
                    // This will setup a fullscreen triangle for the vertex state
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![ShaderDefVal::UInt("NEIGHBOURS_COUNT".into(), 4)],
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
