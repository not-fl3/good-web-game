use cgmath::{Matrix4, Transform, Vector2, Vector4};
use std::path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::{
    error::GameResult,
    filesystem,
    graphics::{context::image_shader, BlendMode, DrawParam, Drawable, Rect},
    Context,
};

use miniquad::{Bindings, Buffer, BufferType, PassAction, Texture};

pub use miniquad::graphics::FilterMode;

#[derive(Debug, Clone)]
#[repr(C)]
pub(crate) struct InstanceAttributes {
    pub source: Vector4<f32>,
    pub color: Vector4<f32>,
    pub model: Matrix4<f32>,
}

impl Default for InstanceAttributes {
    fn default() -> InstanceAttributes {
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
    dirty_filter: Arc<AtomicBool>,

    clones_hack: Arc<()>,
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
            .unwrap_or_else(|_e| panic!("Failed to load png from memory"))
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
        let texture = Texture::from_rgba8(&mut ctx.quad_ctx, width, height, bytes);

        Self::from_texture(&mut ctx.quad_ctx, texture)
    }

    pub fn from_texture(ctx: &mut miniquad::Context, texture: Texture) -> GameResult<Image> {
        #[rustfmt::skip]
        let vertices: [f32; 8] = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![texture],
        };

        Ok(Image {
            width: texture.width as u16,
            height: texture.height as u16,
            texture,
            bindings,
            dirty_filter: Arc::new(AtomicBool::new(false)),
            filter: FilterMode::Linear,
            clones_hack: Arc::new(()),
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
        self.dirty_filter.store(true, Ordering::Release);
        self.filter = filter;
    }

    pub fn filter(&self) -> FilterMode {
        self.filter
    }
}

pub(crate) fn param_to_instance_transform(param: &DrawParam) -> Matrix4<f32> {
    // let size_vec = Vector2::new(param.scale.x, param.scale.y);
    // let size = Matrix4::from_nonuniform_scale(size_vec.x, size_vec.y, 0.);
    // let dest = Point2::new(param.dest.x, param.dest.y);
    // let translate = Matrix4::from_translation(Vector3::new(dest.x, dest.y, 0.));
    // let offset = Matrix4::from_translation(Vector3::new(param.offset.x, param.offset.y, 0.));
    // let offset_inverse =
    //     Matrix4::from_translation(Vector3::new(-param.offset.x, -param.offset.y, 0.));
    // let rot = Matrix4::from_angle_z(cgmath::Rad(param.rotation));
    // //let pos0 = Matrix4::from_translation(Vector3::new(-size_vec.x / 2., -size_vec.y / 2., 0.));

    // let m1 = translate * offset * rot * size * offset_inverse;

    let cosr = param.rotation.cos();
    let sinr = param.rotation.sin();
    let m00 = cosr * param.scale.x;
    let m01 = -sinr * param.scale.y;
    let m10 = sinr * param.scale.x;
    let m11 = cosr * param.scale.y;
    let m03 = param.offset.x * (1.0 - m00) - param.offset.y * m01 + param.dest.x;
    let m13 = param.offset.y * (1.0 - m11) - param.offset.x * m10 + param.dest.y;
    Matrix4::new(
        m00, m10, 0.0, 0.0, m01, m11, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, m03, m13, 0.0, 1.0,
    )
}

impl Drawable for Image {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        let src_width = param.src.w;
        let src_height = param.src.h;

        // We have to mess with the scale to make everything
        // be its-unit-size-in-pixels.
        let real_scale = Vector2::new(
            param.scale.x * src_width * f32::from(self.width),
            param.scale.y * src_height * f32::from(self.height),
        );

        let mut new_param = param;
        new_param.scale = real_scale.into();

        let transform = param_to_instance_transform(&new_param);

        if self.dirty_filter.load(Ordering::Acquire) {
            self.dirty_filter.store(false, Ordering::Release);
            self.texture.set_filter(&mut ctx.quad_ctx, self.filter);
        }

        let pass = ctx.framebuffer();
        ctx.quad_ctx.begin_pass(pass, PassAction::Nothing);
        ctx.quad_ctx.apply_pipeline(&ctx.gfx_context.image_pipeline);
        ctx.quad_ctx.apply_bindings(&self.bindings);

        let uniforms = image_shader::Uniforms {
            projection: ctx.gfx_context.projection,
            model: transform,
            source: Vector4::new(param.src.x, param.src.y, param.src.w, param.src.h),
            color: Vector4::new(param.color.r, param.color.g, param.color.b, param.color.a),
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

impl Drop for Image {
    fn drop(&mut self) {
        if Arc::weak_count(&self.clones_hack) == 1 {
            self.texture.delete();
            self.bindings.index_buffer.delete();
            self.bindings.vertex_buffers[0].delete();
        }
    }
}
