use bevy::{
    core_pipeline::prepass::ViewPrepassTextures,
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_resource::{
            BindGroupDescriptor, BindGroupEntry, BindingResource, Operations, PipelineCache,
            RenderPassColorAttachment, RenderPassDescriptor,
        },
        renderer::RenderContext,
        view::{ExtractedView, ViewTarget, ViewUniforms},
    },
};

use crate::{ConfigBuffer, EdgeDetectionPipeline};

pub struct EdgeDetectionNode {
    query: QueryState<(&'static ViewTarget, &'static ViewPrepassTextures), With<ExtractedView>>,
}

impl EdgeDetectionNode {
    pub const IN_VIEW: &str = "view";
    pub const NAME: &str = "edge_detection";

    pub fn new(world: &mut World) -> Self {
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
                    resource: BindingResource::TextureView(post_process.source),
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
                view: post_process.destination,
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
