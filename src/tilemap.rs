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
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TilePosition {
    pub x: i32,
    pub y: i32,
}

impl TilePosition {
    pub fn pos(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Tile {
    pub index: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Layer {
    tile_set: HashMap<TilePosition, usize>,
}

#[derive(TypeUuid, serde::Serialize, serde::Deserialize, Default, RenderResources)]
#[uuid = "3ab8794d-37ab-4579-adba-a12c290150e9"]
pub struct TileMap {
    #[render_resources(ignore)]
    layers: HashMap<i32, Layer>,
    #[render_resources(ignore)]
    tile_size: Vec2,
    #[render_resources(ignore)]
    changed: bool,
}

#[derive(RenderResources, TypeUuid, serde::Serialize, serde::Deserialize)]
#[uuid = "f2039e1e-64fe-48b1-8b39-5a60ebfce4db"]
pub struct TileSet {
    #[render_resources(ignore)]
    tiles: Vec<Tile>,
}

impl TileMap {
    pub fn generate_mesh(&self, tile_set: &TileSet) -> Mesh {
        let mut mesh = Mesh::new(Default::default());

        let mut verts = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        for (ident, layer) in &self.layers {
            for (pos, tile) in &layer.tile_set {
                let screen = *ISO_TO_SCREEN * pos.pos().extend(0.0);

                let index = verts.len() as u32;

                indices.push(index + 2);
                indices.push(index + 1);
                indices.push(index);
                indices.push(index + 3);
                indices.push(index + 2);
                indices.push(index + 1);

                verts.push([
                    screen.x - self.tile_size.x / 2.0,
                    screen.y - self.tile_size.y / 2.0,
                    *ident as f32 * 1024.0,
                ]);
                verts.push([
                    screen.x + self.tile_size.x / 2.0,
                    screen.y - self.tile_size.y / 2.0,
                    *ident as f32 * 1024.0,
                ]);
                verts.push([
                    screen.x - self.tile_size.x / 2.0,
                    screen.y + self.tile_size.y / 2.0,
                    *ident as f32 * 1024.0,
                ]);
                verts.push([
                    screen.x + self.tile_size.x / 2.0,
                    screen.y + self.tile_size.y / 2.0,
                    *ident as f32 * 1024.0,
                ]);

                normals.push([0.0, 0.0, 1.0]);
                normals.push([0.0, 0.0, 1.0]);
                normals.push([0.0, 0.0, 1.0]);
                normals.push([0.0, 0.0, 1.0]);

                uvs.push([1.0, 1.0]);
                uvs.push([0.0, 1.0]);
                uvs.push([1.0, 0.0]);
                uvs.push([0.0, 0.0]);
            }
        }

        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, verts);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

        mesh
    }
}

pub fn tile_map_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut tile_maps: ResMut<Assets<TileMap>>,
    tile_set: Res<Assets<TileSet>>,
    mut query: Query<(&mut Handle<Mesh>, &Handle<TileMap>, &Handle<TileSet>)>,
) {
    for (mut mesh_handle, tile_map_handle, tile_set_handle) in query.iter_mut() {
        if let Some(tile_map) = tile_maps.get_mut(tile_map_handle) {
            let tile_set = tile_set.get(tile_set_handle).unwrap();

            if !tile_map.changed {
                continue;
            }

            if let Some(mesh) = meshes.get_mut(mesh_handle.clone()) {
                *mesh = tile_map.generate_mesh(tile_set);
            } else {
                *mesh_handle = meshes.add(tile_map.generate_mesh(tile_set));
            }
        }
    }
}

#[derive(Bundle)]
pub struct TileMapBundle {
    pub mesh: Handle<Mesh>,
    pub tile_set: Handle<TileSet>,
    pub material: Handle<ColorMaterial>,
    pub tilemap: Handle<TileMap>,
    pub main_pass: bevy::render::render_graph::base::MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for TileMapBundle {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            tile_set: Default::default(),
            material: Default::default(),
            tilemap: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            visible: Visible {
                is_transparent: true,
                ..Default::default()
            },
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                TILEMAP_PIPELINE_HANDLE.typed(),
            )]),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

pub struct TileMapLoader;

ron_loader!(TileMapLoader, "tile_map" => TileMap, "tile_set" => TileSet);

pub struct TileMapPlugin(bool);

impl TileMapPlugin {
    pub fn server() -> Self {
        Self(true)
    }

    pub fn client() -> Self {
        Self(false)
    }
}

pub const TILEMAP_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 45672853815236);

impl Plugin for TileMapPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.add_asset::<TileMap>();
        app_builder.add_asset::<TileSet>();
        app_builder.add_asset_loader(TileMapLoader);

        if self.0 {
            return;
        }

        app_builder.add_system(tile_map_system.system());

        let resources = app_builder.resources_mut();
        let asset_server = resources.get::<AssetServer>().unwrap();
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();

        render_graph.add_system_node("tile_set", AssetRenderResourcesNode::<TileSet>::new(false));
        render_graph
            .add_node_edge("tile_set", base::node::MAIN_PASS)
            .unwrap();

        render_graph.add_system_node("tile_map", AssetRenderResourcesNode::<TileMap>::new(true));
        render_graph
            .add_node_edge("tile_map", base::node::MAIN_PASS)
            .unwrap();

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
                vertex: asset_server.load("shaders/tilemap_shader.vert"),
                fragment: Some(asset_server.load("shaders/tilemap_shader.frag")),
            })
        };

        let mut pipelines = resources.get_mut::<Assets<PipelineDescriptor>>().unwrap();

        pipelines.set_untracked(TILEMAP_PIPELINE_HANDLE, pipeline);
    }
}
