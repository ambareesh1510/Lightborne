use bevy::{
    ecs::{
        query::{QueryItem, ROQueryItem},
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    math::{vec2, vec3, Affine3},
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        mesh::VertexBufferLayout,
        render_phase::{PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::{binding_types::uniform_buffer, *},
        renderer::{RenderDevice, RenderQueue},
        view::ViewTarget,
        Render, RenderApp, RenderSet,
    },
    sprite::Mesh2dPipeline,
};
use bytemuck::{Pod, Zeroable};

use super::render::PostProcessRes;

pub struct PointLight2dPlugin;

impl Plugin for PointLight2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<PointLight2d>::default())
            .add_plugins(UniformComponentPlugin::<ExtractPointLight2d>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.add_systems(
            Render,
            prepare_point_light_2d_bind_group.in_set(RenderSet::PrepareBindGroups),
        );
    }
    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<PointLight2dPipeline>()
            .init_resource::<PointLight2dBuffers>();
    }
}

#[derive(Component, Default)]
#[require(Transform)]
pub struct PointLight2d {
    pub color: Vec4,
    pub half_length: f32,
    pub radius: f32,
    pub volumetric_intensity: f32,
}

impl ExtractComponent for PointLight2d {
    type Out = (ExtractPointLight2d, PointLight2dBounds);
    type QueryData = (&'static GlobalTransform, &'static PointLight2d);
    type QueryFilter = ();

    fn extract_component(
        (transform, point_light): QueryItem<'_, Self::QueryData>,
    ) -> Option<Self::Out> {
        // FIXME: don't do computations in extract
        let affine_a = transform.affine();
        let affine = Affine3::from(&affine_a);
        let (a, b) = affine.inverse_transpose_3x3();

        Some((
            ExtractPointLight2d {
                world_from_local: affine.to_transpose(),
                local_from_world_transpose_a: a,
                local_from_world_transpose_b: b,
                color: point_light.color,
                half_length: point_light.half_length,
                radius: point_light.radius,
                volumetric_intensity: point_light.volumetric_intensity,
            },
            PointLight2dBounds {
                transform: transform.compute_transform(),
                half_length: point_light.half_length,
                radius: point_light.radius,
            },
        ))
    }
}

/// Render world version of [`PointLight2d`].  
#[derive(Component, ShaderType, Clone, Copy, Debug)]
pub struct ExtractPointLight2d {
    world_from_local: [Vec4; 3],
    local_from_world_transpose_a: [Vec4; 2],
    local_from_world_transpose_b: f32,
    color: Vec4,
    pub half_length: f32,
    pub radius: f32,
    volumetric_intensity: f32,
}

#[derive(Component, Clone, Copy)]
pub struct PointLight2dBounds {
    pub transform: Transform,
    pub radius: f32,
    pub half_length: f32,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct PointLight2dVertex {
    position: Vec3,
    uv: Vec2,
    /// 0 -> inner, 1 -> outer
    variant: u32,
}

impl PointLight2dVertex {
    const fn inner(position: Vec3, uv: Vec2) -> Self {
        PointLight2dVertex {
            position,
            uv,
            variant: 0,
        }
    }
    const fn outer(position: Vec3, uv: Vec2) -> Self {
        PointLight2dVertex {
            position,
            uv,
            variant: 1,
        }
    }
}

#[derive(Resource)]
pub struct PointLight2dBuffers {
    pub vertices: RawBufferVec<PointLight2dVertex>,
    pub indices: RawBufferVec<u32>,
}

pub const POINT_LIGHT_2D_NUM_INDICES: u32 = 18;

static VERTICES: [PointLight2dVertex; 8] = [
    PointLight2dVertex::inner(vec3(-1.0, -1.0, 0.0), vec2(0.5, 0.0)),
    PointLight2dVertex::inner(vec3(1.0, -1.0, 0.0), vec2(0.5, 0.0)),
    PointLight2dVertex::inner(vec3(1.0, 1.0, 0.0), vec2(0.5, 1.0)),
    PointLight2dVertex::inner(vec3(-1.0, 1.0, 0.0), vec2(0.5, 1.0)),
    PointLight2dVertex::outer(vec3(-1.0, -1.0, 0.0), vec2(0.0, 0.0)),
    PointLight2dVertex::outer(vec3(1.0, -1.0, 0.0), vec2(1.0, 0.0)),
    PointLight2dVertex::outer(vec3(1.0, 1.0, 0.0), vec2(1.0, 1.0)),
    PointLight2dVertex::outer(vec3(-1.0, 1.0, 0.0), vec2(0.0, 1.0)),
];

static INDICES: [u32; 18] = [0, 1, 2, 2, 3, 0, 1, 5, 6, 6, 2, 1, 4, 0, 3, 3, 7, 4];

impl FromWorld for PointLight2dBuffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let mut vbo = RawBufferVec::new(BufferUsages::VERTEX);
        let mut ibo = RawBufferVec::new(BufferUsages::INDEX);

        for vtx in &VERTICES {
            vbo.push(*vtx);
        }
        for index in &INDICES {
            ibo.push(*index);
        }

        vbo.write_buffer(render_device, render_queue);
        ibo.write_buffer(render_device, render_queue);

        PointLight2dBuffers {
            vertices: vbo,
            indices: ibo,
        }
    }
}

pub fn point_light_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(
        "point_light_bind_group_layout",
        &BindGroupLayoutEntries::single(
            ShaderStages::VERTEX_FRAGMENT,
            uniform_buffer::<ExtractPointLight2d>(true),
        ),
    )
}

#[derive(Resource)]
pub struct PointLight2dBindGroup {
    value: BindGroup,
}

pub fn prepare_point_light_2d_bind_group(
    mut commands: Commands,
    uniforms: Res<ComponentUniforms<ExtractPointLight2d>>,
    pipeline: Res<PointLight2dPipeline>,
    render_device: Res<RenderDevice>,
) {
    if let Some(binding) = uniforms.uniforms().binding() {
        commands.insert_resource(PointLight2dBindGroup {
            value: render_device.create_bind_group(
                "point_light_2d_bind_group",
                &pipeline.layout,
                &BindGroupEntries::single(binding),
            ),
        })
    }
}

pub struct SetPointLight2dBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetPointLight2dBindGroup<I> {
    type Param = SRes<PointLight2dBindGroup>;
    type ViewQuery = ();
    type ItemQuery = Read<DynamicUniformIndex<ExtractPointLight2d>>;

    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(index) = entity else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &param.into_inner().value, &[index.index()]);
        RenderCommandResult::Success
    }
}

pub struct DrawPointLight2d;
impl<P: PhaseItem> RenderCommand<P> for DrawPointLight2d {
    type Param = SRes<PointLight2dBuffers>;
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let buffers = param.into_inner();

        pass.set_stencil_reference(0); // only render if no occluders here

        pass.set_vertex_buffer(0, buffers.vertices.buffer().unwrap().slice(..));
        pass.set_index_buffer(
            buffers.indices.buffer().unwrap().slice(..),
            0,
            IndexFormat::Uint32,
        );
        pass.draw_indexed(0..POINT_LIGHT_2D_NUM_INDICES, 0, 0..1);

        RenderCommandResult::Success
    }
}

#[derive(Resource)]
pub struct PointLight2dPipeline {
    pub layout: BindGroupLayout,
    pub pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for PointLight2dPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let post_process_res = world.resource::<PostProcessRes>();
        let post_process_layout = post_process_res.layout.clone();

        let layout = point_light_bind_group_layout(render_device);

        let shader = world.load_asset("shaders/lighting/point_light.wgsl");

        let pos_buffer_layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<PointLight2dVertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: vec![
                // Position
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: std::mem::offset_of!(PointLight2dVertex, position) as u64,
                    shader_location: 0,
                },
                // UV
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: std::mem::offset_of!(PointLight2dVertex, uv) as u64,
                    shader_location: 1,
                },
                // Variant (Inner vs Outer vertex)
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: std::mem::offset_of!(PointLight2dVertex, variant) as u64,
                    shader_location: 2,
                },
            ],
        };

        let mesh2d_pipeline = Mesh2dPipeline::from_world(world);

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("point_light_pipeline".into()),
                    layout: vec![
                        post_process_layout,
                        mesh2d_pipeline.view_layout,
                        layout.clone(),
                    ],
                    vertex: VertexState {
                        shader: shader.clone(),
                        shader_defs: vec![],
                        entry_point: "vertex".into(),
                        buffers: vec![pos_buffer_layout],
                    },
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: ViewTarget::TEXTURE_FORMAT_HDR,
                            blend: Some(BlendState {
                                color: BlendComponent {
                                    src_factor: BlendFactor::One,
                                    dst_factor: BlendFactor::One,
                                    operation: BlendOperation::Add,
                                },
                                alpha: BlendComponent::OVER,
                            }),
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    // below needs changing?
                    primitive: PrimitiveState::default(),
                    depth_stencil: Some(DepthStencilState {
                        format: TextureFormat::Stencil8,
                        depth_write_enabled: false,
                        depth_compare: CompareFunction::Always,
                        stencil: StencilState {
                            front: StencilFaceState {
                                compare: CompareFunction::Equal,
                                fail_op: StencilOperation::Keep,
                                depth_fail_op: StencilOperation::Keep,
                                pass_op: StencilOperation::Keep,
                            },
                            back: StencilFaceState::default(),
                            read_mask: 0xFF,
                            write_mask: 0xFF,
                        },
                        bias: DepthBiasState::default(),
                    }),
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                    zero_initialize_workgroup_memory: false,
                });

        PointLight2dPipeline {
            layout,
            pipeline_id,
        }
    }
}
