use bevy::{
    asset::embedded_asset, core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        fullscreen_vertex_shader::fullscreen_shader_vertex_state, prepass::{DepthPrepass, NormalPrepass, ViewPrepassTextures},
    }, ecs::query::QueryItem, prelude::*, render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, texture_depth_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::{ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
        RenderApp,
    }
};


pub struct SimpletoonPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SimpletoonPostProcessLabel;

#[derive(Default)]
struct SimpletoonPostProcessNode;

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
#[require(DepthPrepass, NormalPrepass)]
pub struct SimpletoonSettings {
    pub depth_threshold: f32,
    pub depth_threshold_depth_mul: f32,  // If something is further away, it should require more depth
    pub depth_normal_threshold: f32, // If at a glazing angle, depth threshold should be harsher
    pub depth_normal_threshold_mul: f32, // If at a glazing angle, depth threshold should be harsher
    pub normal_threshold: f32,
    pub colour_threshold: f32,
    pub stroke_size: f32,
    pub colour_banding: f32,
    pub stroke_colour: Vec4
}

#[derive(Resource)]
struct PostProcessPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl Plugin for SimpletoonPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "assets/toon.wgsl");
        app.add_plugins((
            ExtractComponentPlugin::<SimpletoonSettings>::default(),
            UniformComponentPlugin::<SimpletoonSettings>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_graph_node::<ViewNodeRunner<SimpletoonPostProcessNode>>(
                Core3d,
                SimpletoonPostProcessLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::Tonemapping,
                    SimpletoonPostProcessLabel,
                    Node3d::Fxaa,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<PostProcessPipeline>();
    }
}

impl ViewNode for SimpletoonPostProcessNode {
    // The node needs a query to gather data from the ECS in order to do its rendering,
    // but it's not a normal system so we need to define it manually.
    //
    // This query will only run on the view entity
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewPrepassTextures,
        // This makes sure the node only runs on cameras with the PostProcessSettings component
        &'static SimpletoonSettings,
        // As there could be multiple post processing components sent to the GPU (one per camera),
        // we need to get the index of the one that is associated with the current view.
        &'static DynamicUniformIndex<SimpletoonSettings>,
        &'static ViewUniformOffset,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, prepass_textures, _post_process_settings, settings_index, view_uniform): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {

        let post_process_pipeline = world.resource::<PostProcessPipeline>();

        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<SimpletoonSettings>>();
        let view_uniforms = world.resource::<ViewUniforms>();
        let Some(view_uniforms) = view_uniforms.uniforms.binding() else {
            return Ok(());
        };
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };
        let (Some(depth_texture), Some(normal_texture)) =
            (&prepass_textures.depth, &prepass_textures.normal)
        else {
            println!("could not find depth or normal");
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "post_process_bind_group",
            &post_process_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &post_process_pipeline.sampler,
                settings_binding.clone(),
                &depth_texture.texture.default_view,
                &normal_texture.texture.default_view,
                view_uniforms
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);

        render_pass.set_bind_group(0, &bind_group, &[settings_index.index(), view_uniform.offset]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

impl FromWorld for PostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "post_process_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<SimpletoonSettings>(true),
                    texture_depth_2d(),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    uniform_buffer::<ViewUniform>(true),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world.load_asset("embedded://bevy_simpletoon/assets/toon.wgsl");

        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("post_process_pipeline".into()),
                layout: vec![layout.clone()],
                vertex: fullscreen_shader_vertex_state(),
                fragment: Some(FragmentState {
                    shader,
                    shader_defs: vec![],
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
                zero_initialize_workgroup_memory: false,
            });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}

impl Default for SimpletoonSettings {
    fn default() -> Self {
        Self { 
            depth_threshold: 1.0, 
            depth_threshold_depth_mul: 1.0, 
            depth_normal_threshold: 0.4, 
            depth_normal_threshold_mul: 30.0, 
            normal_threshold: 0.4, 
            colour_threshold: 0.2, 
            stroke_size: 1.0,
            colour_banding: 5.0, 
            stroke_colour: Vec4::new(0.1, 0.1, 0.1, 1.0) 
        }
    }
}