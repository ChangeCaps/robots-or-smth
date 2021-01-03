use crate::*;
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline, *},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::ShaderStages,
        texture::TextureFormat,
    },
};

pub const SPRITE_SHEET_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 7912634715172);
pub const SPRITE_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 45672853815236);

pub struct SpriteShaderPlugin;

impl Plugin for SpriteShaderPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        let resources = app_builder.resources_mut();
        let asset_server = resources.get::<AssetServer>().unwrap();
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();

        render_graph.add_system_node("tile_map", AssetRenderResourcesNode::<TileMap>::new(true));
        render_graph
            .add_node_edge("tile_map", base::node::MAIN_PASS)
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
                vertex: asset_server.load("shaders/sprite_sheet_shader.vert"),
                fragment: Some(asset_server.load("shaders/sprite_sheet_shader.frag")),
            })
        };

        pipelines.set_untracked(SPRITE_SHEET_PIPELINE_HANDLE, pipeline);

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
                vertex: asset_server.load("shaders/sprite_shader.vert"),
                fragment: Some(asset_server.load("shaders/sprite_shader.frag")),
            })
        };

        pipelines.set_untracked(SPRITE_PIPELINE_HANDLE, pipeline);
    }
}
