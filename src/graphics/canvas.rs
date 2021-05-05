use crate::{
    conf::NumSamples,
    graphics::{BlendMode, DrawParam, Drawable, FilterMode, Image, Rect},
    Context, GameResult,
};

use miniquad::{RenderPass, Texture, TextureFormat, TextureParams};

#[derive(Clone, Debug)]
pub struct Canvas {
    image: Image,
    pub(crate) offscreen_pass: RenderPass,
}

impl Canvas {
    pub fn new(
        ctx: &mut Context,
        width: u16,
        height: u16,
        _samples: NumSamples,
    ) -> GameResult<Canvas> {
        let texture = Texture::new_render_texture(
            &mut ctx.quad_ctx,
            TextureParams {
                width: width as u32,
                height: height as u32,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );

        let image = Image::from_texture(&mut ctx.quad_ctx, texture)?;

        let offscreen_pass = RenderPass::new(&mut ctx.quad_ctx, texture, None);

        Ok(Canvas {
            image,
            offscreen_pass,
        })
    }

    /// Create a new `Canvas` with the current window dimensions.
    pub fn with_window_size(ctx: &mut Context) -> GameResult<Canvas> {
        use crate::graphics;
        let (w, h) = graphics::drawable_size(ctx);
        // Default to no multisampling
        // TODO: Use winit's into() to translate f64's more accurately
        // ...where the heck IS winit's into()?  wth was I referring to?
        Canvas::new(ctx, w as u16, h as u16, NumSamples::One)
    }

    /// Gets the backend `Image` that is being rendered to.
    pub fn image(&self) -> &Image {
        &self.image
    }

    /// Get the filter mode for the image.
    pub fn filter(&self) -> FilterMode {
        self.image.filter()
    }

    /// Set the filter mode for the canvas.
    pub fn set_filter(&mut self, mode: FilterMode) {
        self.image.set_filter(mode)
    }

    /// Destroys the `Canvas` and returns the `Image` it contains.
    pub fn into_inner(self) -> Image {
        // TODO: This texture is created with different settings
        // than the default; does that matter?
        self.image
    }
}

impl Drawable for Canvas {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        // Gotta flip the image on the Y axis here
        // to account for OpenGL's origin being at the bottom-left.
        let mut flipped_param = param;
        flipped_param.scale.y *= -1.0;
        flipped_param.dest.y += self.image.height() as f32 * param.scale.y;
        self.image.draw(ctx, flipped_param)
    }

    fn set_blend_mode(&mut self, blend_mode: Option<BlendMode>) {
        self.image.set_blend_mode(blend_mode);
    }

    fn blend_mode(&self) -> Option<BlendMode> {
        self.image.blend_mode()
    }

    fn dimensions(&self, _: &mut Context) -> Option<Rect> {
        Some(self.image.dimensions())
    }
}

/// Set the `Canvas` to render to. Specifying `Option::None` will cause all
/// rendering to be done directly to the screen.
pub fn set_canvas(ctx: &mut Context, target: Option<&Canvas>) {
    ctx.gfx_context.canvas = target.cloned();
}
