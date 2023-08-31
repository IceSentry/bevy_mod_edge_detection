use bevy::{
    core_pipeline::prepass::ViewPrepassTextures,
    prelude::*,
    render::{
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        render_resource::{
            Operations, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor,
        },
        renderer::RenderContext,
        view::{ViewTarget, ViewUniforms},
    },
};

use crate::{
    render_ext::{BindingResouceExt, RenderDeviceExt},
    ConfigBuffer, EdgeDetectionPipeline,
};

#[derive(Default)]
pub struct EdgeDetectionNode;
impl EdgeDetectionNode {
    pub const NAME: &str = "edge_detection_node";
}

impl ViewNode for EdgeDetectionNode {
    type ViewQuery = (&'static ViewTarget, &'static ViewPrepassTextures);

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, prepass_textures): bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let edge_detection_pipeline = world.resource::<EdgeDetectionPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(pipeline) =
            pipeline_cache.get_render_pipeline(edge_detection_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();
        let view_uniforms = world.resource::<ViewUniforms>();
        let config_buffer = world.resource::<ConfigBuffer>();

        let bind_group = render_context.render_device().create_bind_group_ext(
            "edge_detection_bind_group",
            &edge_detection_pipeline.layout,
            [
                post_process.source.bind(),
                edge_detection_pipeline.sampler.bind(),
                prepass_textures.depth.bind(),
                prepass_textures.normal.bind(),
                view_uniforms.uniforms.bind(),
                config_buffer.buffer.bind(),
            ],
        );

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
