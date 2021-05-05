use crate::{
    graphics::{types::Rect, Canvas},
    GameResult,
};
use miniquad_text_rusttype::FontAtlas;
use miniquad_text_rusttype::FontTexture;
use std::rc::Rc;

use cgmath::{Matrix3, Matrix4};

const DEFAULT_FONT_BYTES: &'static [u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/DejaVuSerif.ttf"
));

pub struct GraphicsContext {
    pub(crate) screen_rect: Rect,
    pub(crate) projection: Matrix4<f32>,
    pub(crate) white_texture: miniquad::Texture,
    pub(crate) canvas: Option<Canvas>,
    pub(crate) sprite_pipeline: miniquad::Pipeline,
    pub(crate) mesh_pipeline: miniquad::Pipeline,
    pub(crate) image_pipeline: miniquad::Pipeline,
    pub(crate) text_system: miniquad_text_rusttype::TextSystem,
    pub(crate) fonts_cache: Vec<Rc<miniquad_text_rusttype::FontTexture>>,
    pub(crate) font_size: u32,
}

impl GraphicsContext {
    pub fn new(ctx: &mut miniquad::Context) -> GraphicsContext {
        use miniquad::*;

        let projection = cgmath::One::one();
        let screen_rect = Rect::new(-1., -1., 2., 2.);

        let white_texture = Texture::from_rgba8(ctx, 1, 1, &[255, 255, 255, 255]);

        let sprite_shader = Shader::new(
            ctx,
            batch_shader::VERTEX,
            batch_shader::FRAGMENT,
            batch_shader::META,
        );

        let sprite_pipeline = miniquad::Pipeline::with_params(
            ctx,
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
                VertexAttribute::with_buffer("InstanceModel", VertexFormat::Mat4, 1),
            ],
            sprite_shader,
            PipelineParams {
                color_blend: Some((
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );

        let image_shader = Shader::new(
            ctx,
            image_shader::VERTEX,
            image_shader::FRAGMENT,
            image_shader::META,
        );

        let image_pipeline = miniquad::Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[VertexAttribute::with_buffer(
                "position",
                VertexFormat::Float2,
                0,
            )],
            image_shader,
            PipelineParams {
                color_blend: Some((
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );

        let mesh_shader = Shader::new(
            ctx,
            mesh_shader::VERTEX,
            mesh_shader::FRAGMENT,
            mesh_shader::META,
        );

        let mesh_pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("position", VertexFormat::Float2),
                VertexAttribute::new("texcoord", VertexFormat::Float2),
                VertexAttribute::new("color0", VertexFormat::Float4),
            ],
            mesh_shader,
            PipelineParams {
                color_blend: Some((
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );

        let text_system = miniquad_text_rusttype::TextSystem::new(ctx);

        // load default font, will be available by FontId::default()
        let fonts_cache = vec![Rc::new(load_font(ctx, DEFAULT_FONT_BYTES, 70).unwrap())];

        GraphicsContext {
            projection,
            screen_rect,
            white_texture,
            canvas: None,
            sprite_pipeline,
            mesh_pipeline,
            image_pipeline,
            text_system,
            fonts_cache,
            font_size: 50,
        }
    }
}

impl GraphicsContext {
    pub(crate) fn load_font(
        &mut self,
        ctx: &mut miniquad::Context,
        font_bytes: &[u8],
        font_size: u32,
    ) -> GameResult<usize> {
        let font = load_font(ctx, &font_bytes, font_size)?;

        self.fonts_cache.push(Rc::new(font));

        Ok(self.fonts_cache.len() - 1)
    }

    pub fn set_transform(&mut self, _transform: &Matrix3<f32>) {
        unimplemented!();
    }

    pub fn push_transform(&mut self, _transform: &Matrix3<f32>) {
        unimplemented!();
    }

    pub fn pop_transform(&mut self) {
        unimplemented!();
    }

    pub fn set_screen_coordinates(&mut self, rect: crate::graphics::types::Rect) {
        self.screen_rect = rect;
        self.projection =
            cgmath::ortho(rect.x, rect.x + rect.w, rect.y + rect.h, rect.y, -1.0, 1.0);
    }
}

fn load_font(
    ctx: &mut miniquad::Context,
    font_data: &[u8],
    font_size: u32,
) -> GameResult<FontTexture> {
    Ok(FontTexture::new(
        ctx,
        font_data,
        font_size,
        FontAtlas::ascii_character_list(),
    )?)
}

pub(crate) mod batch_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;
    attribute vec4 Source;
    attribute vec4 Color;
    attribute mat4 InstanceModel;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 Projection;
    uniform mat4 Model;

    uniform float depth;

    void main() {
        gl_Position = Projection * Model * InstanceModel * vec4(position, 0, 1);
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
            uniforms: &[
                ("Projection", UniformType::Mat4),
                ("Model", UniformType::Mat4),
            ],
        },
    };

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub projection: cgmath::Matrix4<f32>,
        pub model: cgmath::Matrix4<f32>,
    }
}

pub(crate) mod image_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 Projection;
    uniform vec4 Source;
    uniform vec4 Color;
    uniform mat4 Model;

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
            uniforms: &[
                ("Projection", UniformType::Mat4),
                ("Source", UniformType::Float4),
                ("Color", UniformType::Float4),
                ("Model", UniformType::Mat4),
            ],
        },
    };

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub projection: cgmath::Matrix4<f32>,
        pub source: cgmath::Vector4<f32>,
        pub color: cgmath::Vector4<f32>,
        pub model: cgmath::Matrix4<f32>,
    }
}

pub(crate) mod mesh_shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform mat4 Projection;
    uniform mat4 Model;
    uniform vec4 Color;

    uniform float depth;

    void main() {
        gl_Position = Projection * Model * vec4(position, 0, 1);
        gl_Position.z = depth;
        color = Color * color0;
        uv = texcoord;
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
            uniforms: &[
                ("Projection", UniformType::Mat4),
                ("Model", UniformType::Mat4),
                ("Color", UniformType::Float4),
            ],
        },
    };

    #[repr(C)]
    #[derive(Debug)]
    pub struct Uniforms {
        pub projection: cgmath::Matrix4<f32>,
        pub model: cgmath::Matrix4<f32>,
        pub color: cgmath::Vector4<f32>,
    }
}
