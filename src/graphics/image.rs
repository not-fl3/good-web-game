use cgmath::{Matrix4, Point2, Vector2, Vector3, Vector4};
use std::{path};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    error::GameResult,
    filesystem,
    graphics::{BlendMode, DrawParam, Drawable, Rect},
    Context,
};

use miniquad::{
    Bindings, BlendFactor, BlendValue, Buffer, BufferLayout, BufferType, Equation, PassAction,
    Pipeline, PipelineParams, Shader, Texture, VertexAttribute, VertexFormat, VertexStep,
};

#[derive(Debug, Clone)]
#[repr(C)]
pub(crate) struct InstanceAttributes {
    pub source: Vector4<f32>,
    pub color: Vector4<f32>,
    pub model: Matrix4<f32>,
}

impl Default for InstanceAttributes {
    fn default() -> InstanceAttributes {
        use cgmath::Transform;
        InstanceAttributes {
            source: Vector4::new(0., 0., 0., 0.),
            color: Vector4::new(0., 0., 0., 0.),
            model: Matrix4::one(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Image {
    pub(crate) texture: Texture,
    pub(crate) width: u16,
    pub(crate) height: u16,
    filter: FilterMode,
    pub(crate) bindings: Bindings,
    pub(crate) pipeline: Pipeline,
    dirty_filter: DirtyFlag,
}

#[derive(Clone, Copy, Debug)]
pub enum FilterMode {
    Linear,  // = 0LINEAR_FILTER as isize,
    Nearest, // = NEAREST_FILTER as isize,
}

impl Image {
    pub fn new<P: AsRef<path::Path>>(ctx: &mut Context, path: P) -> GameResult<Self> {
        use std::io::Read;

        let mut file = filesystem::open(ctx, path)?;

        let mut bytes = vec![];
        file.bytes.read_to_end(&mut bytes)?;

        Self::from_png_bytes(ctx, &bytes)
    }

    pub fn from_png_bytes(ctx: &mut Context, bytes: &[u8]) -> GameResult<Self> {
        let img = image::load_from_memory(&bytes)
            .unwrap_or_else(|e| panic!(e))
            .to_rgba();
        let width = img.width() as u16;
        let height = img.height() as u16;
        let bytes = img.into_raw();

        Image::from_rgba8(ctx, width, height, &bytes)
    }

    pub fn from_rgba8(
        ctx: &mut Context,
        width: u16,
        height: u16,
        bytes: &[u8],
    ) -> GameResult<Image> {
        let texture = Texture::from_rgba8(width, height, bytes);

        Self::from_texture(ctx, texture)
    }

    pub fn from_texture(ctx: &mut Context, texture: Texture) -> GameResult<Image> {
        #[rustfmt::skip]
        let vertices: [f32; 8] = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
        let vertex_buffer =
            Buffer::immutable(&mut ctx.quad_ctx, BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(&mut ctx.quad_ctx, BufferType::IndexBuffer, &indices);

        let instances_buffer = Buffer::stream(
            &mut ctx.quad_ctx,
            BufferType::VertexBuffer,
            std::mem::size_of::<InstanceAttributes>(),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer, instances_buffer],
            index_buffer: index_buffer,
            images: vec![texture],
        };

        let shader = Shader::new(
            &mut ctx.quad_ctx,
            batch_shader::VERTEX,
            batch_shader::FRAGMENT,
            batch_shader::META,
        );

        let pipeline = Pipeline::with_params(
            &mut ctx.quad_ctx,
            &[
                BufferLayout::default(),
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            &[
                VertexAttribute::with_buffer("position", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("Source", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("Color", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("Model", VertexFormat::Mat4, 1),
            ],
            shader,
            PipelineParams {
                color_blend: Some((
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );

        Ok(Image {
            width: texture.width as u16,
            height: texture.height as u16,
            texture,
            bindings,
            pipeline,
            dirty_filter: DirtyFlag::new(false),
            filter: FilterMode::Linear,
        })
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    /// Returns the dimensions of the image.
    pub fn dimensions(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width() as f32, self.height() as f32)
    }

    pub fn set_filter(&mut self, filter: FilterMode) {
        self.dirty_filter.store(true);
        self.filter = filter;
    }

    pub fn filter(&self) -> FilterMode {
        self.filter
    }
}

pub(crate) fn param_to_instance_transform(
    param: &DrawParam,
    width: u16,
    height: u16,
) -> Matrix4<f32> {
    let real_size = Vector2::new(param.src.w * width as f32, param.src.h * height as f32);
    let size_vec = Vector2::new(real_size.x * param.scale.x, real_size.y * param.scale.y);
    let size = Matrix4::from_nonuniform_scale(size_vec.x, size_vec.y, 0.);
    let dest = Point2::new(
        param.dest.x - real_size.x * param.offset.x * param.scale.x,
        param.dest.y - real_size.y * param.offset.y * param.scale.y,
    );
    let pos = Matrix4::from_translation(Vector3::new(
        dest.x + size_vec.x / 2.,
        dest.y + size_vec.y / 2.,
        0.,
    ));
    let rot = Matrix4::from_angle_z(cgmath::Rad(param.rotation));
    let pos0 = Matrix4::from_translation(Vector3::new(-size_vec.x / 2., -size_vec.y / 2., 0.));

    pos * rot * pos0 * size
}

impl Drawable for Image {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        let transform = param_to_instance_transform(&param, self.width, self.height);

        if self.dirty_filter.load() {
            self.dirty_filter.store(false);
            self.texture.set_filter(self.filter as i32);
        }

        let instances = &[InstanceAttributes {
            model: transform,
            source: Vector4::new(param.src.x, param.src.y, param.src.w, param.src.h),
            color: Vector4::new(param.color.r, param.color.g, param.color.b, param.color.a),
        }];
        self.bindings.vertex_buffers[1].update(ctx.quad_ctx, instances);

        let pass = ctx.framebuffer();
        ctx.quad_ctx.begin_pass(pass, PassAction::Nothing);
        ctx.quad_ctx.apply_pipeline(&self.pipeline);
        ctx.quad_ctx.apply_bindings(&self.bindings);

        let uniforms = batch_shader::Uniforms {
            projection: ctx.internal.gfx_context.projection,
        };

        ctx.quad_ctx.apply_uniforms(&uniforms);
        ctx.quad_ctx.draw(0, 6, 1);

        ctx.quad_ctx.end_render_pass();

        Ok(())
    }

    fn set_blend_mode(&mut self, _: Option<BlendMode>) {}

    /// Gets the blend mode to be used when drawing this drawable.
    fn blend_mode(&self) -> Option<BlendMode> {
        unimplemented!()
    }

    fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> {
        Some(self.dimensions())
    }
}
pub(crate) mod batch_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;
    attribute vec4 Source;
    attribute vec4 Color;
    attribute mat4 Model;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 Projection;
    
    uniform float depth;

    void main() {
        gl_Position = Projection * Model * vec4(position, 0, 1);
        gl_Position.z = depth;
        color = Color;
        uv = position * Source.zw + Source.xy;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform sampler2D Texture;


    void main() {
        gl_FragColor = texture2D(Texture, uv) * color;
    }"#;

    pub const META: ShaderMeta = ShaderMeta {
        images: &["Texture"],
        uniforms: UniformBlockLayout {
            uniforms: &[("Projection", UniformType::Mat4)],
        },
    };

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub projection: cgmath::Matrix4<f32>,
    }
}

#[derive(Debug)]
struct DirtyFlag(AtomicBool);

impl DirtyFlag {
    pub fn new(value: bool) -> Self {
        Self(AtomicBool::new(value))
    }

    pub fn load(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    
    pub fn store(&self, value: bool) {
        self.0.store(value, Ordering::Release)
    }
}

impl Clone for DirtyFlag {
    fn clone(&self) -> Self {
        DirtyFlag(AtomicBool::new(self.0.load(Ordering::Acquire)))
    }
}
