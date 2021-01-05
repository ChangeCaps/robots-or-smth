use crate::*;
use bevy::{
    reflect::TypeUuid,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline, *},
        render_graph::{base, RenderGraph, RenderResourcesNode},
        renderer::RenderResources,
        shader::ShaderStages,
        texture::TextureFormat,
    },
};

pub const BAR_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 1823518765843);

#[derive(RenderResources, TypeUuid)]
#[uuid = "980be34c-961e-4ac5-ab50-92cba0f8df01"]
pub struct Bar {
    pub size: Vec2,
    pub max_value: f32,
    pub current_value: f32,
    pub border_thickness: f32,
    pub background_color: Color,
    pub color_a: Color,
    pub color_b: Color,
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            size: Vec2::new(64.0, 12.0),
            max_value: 200.0,
            current_value: 200.0,
            border_thickness: 1.0,
            background_color: Color::hex("727272").unwrap(),
            color_a: Color::hex("d63131").unwrap(),
            color_b: Color::hex("cc5f5f").unwrap(),
        }
    }
}

#[derive(Bundle)]
pub struct BarBundle {
    pub bar: Bar,
    pub mesh: Handle<Mesh>,
    pub main_pass: base::MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for BarBundle {
    fn default() -> Self {
        Self {
            bar: Bar::default(),
            mesh: bevy::sprite::QUAD_HANDLE.typed(),
            main_pass: base::MainPass,
            draw: Default::default(),
            visible: Visible {
                is_transparent: true,
                ..Default::default()
            },
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                BAR_PIPELINE_HANDLE.typed(),
            )]),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

pub struct BarPlugin;

impl Plugin for BarPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        let resources = app_builder.resources_mut();
        let shaders = resources.get::<AssetServer>().unwrap();
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();

        render_graph.add_system_node("bar", RenderResourcesNode::<Bar>::new(true));
        render_graph
            .add_node_edge("bar", base::node::MAIN_PASS)
            .unwrap();

        let mut pipelines = resources.get_mut::<Assets<PipelineDescriptor>>().unwrap();

        let pipeline = PipelineDescriptor {
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            depth_stencil_state: Some(DepthStencilStateDescriptor {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil: StencilStateDescriptor {
                    front: StencilStateFaceDescriptor::IGNORE,
                    back: StencilStateFaceDescriptor::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
            }),
            color_states: vec![ColorStateDescriptor {
                format: TextureFormat::default(),
                color_blend: BlendDescriptor {
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                    operation: BlendOperation::Add,
                },
                alpha_blend: BlendDescriptor {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                write_mask: ColorWrite::ALL,
            }],
            ..PipelineDescriptor::new(ShaderStages {
                vertex: shaders.get_handle("shaders/bar_shader.vert"),
                fragment: Some(shaders.get_handle("shaders/bar_shader.frag")),
            })
        };

        pipelines.set_untracked(BAR_PIPELINE_HANDLE, pipeline);
    }
}
