use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    audio,
    conf::Conf,
    filesystem::Filesystem,
    graphics,
    input::{input_handler::InputHandler, KeyboardContext, MouseContext},
    timer::TimeContext,
};

/// A `Context` is an object that holds on to global resources.
/// It basically tracks hardware state such as the screen, audio
/// system, timers, and so on.  Generally this type can **not**
/// be shared/sent between threads and only one `Context` can exist at a time.  Trying
/// to create a second one will fail.  It is fine to drop a `Context`
/// and create a new one, but this will also close and re-open your
/// game's window.
///
/// Most functions that interact with the hardware, for instance
/// drawing things, playing sounds, or loading resources (which then
/// need to be transformed into a format the hardware likes) will need
/// to access the `Context`.  It is an error to create some type that
/// relies upon a `Context`, such as `Image`, and then drop the `Context`
/// and try to draw the old `Image` with the new `Context`.  Most types
/// include checks to make this panic in debug mode, but it's not perfect.
///
/// All fields in this struct are basically undocumented features,
/// only here to make it easier to debug, or to let advanced users
/// hook into the guts of ggez and make it do things it normally
/// can't.  Most users shouldn't need to touch these things directly,
/// since implementation details may change without warning.  The
/// public and stable API is `ggez`'s module-level functions and
/// types.
pub struct Context {
    /// Filesystem state
    pub filesystem: Filesystem,
    /// Audio context
    pub audio_context: audio::AudioContext,
    /// Graphics state
    pub gfx_context: graphics::GraphicsContext,
    /// Mouse context
    pub mouse_context: MouseContext,
    /// Keyboard context
    pub keyboard_context: KeyboardContext,
    /// Timer state
    pub timer_context: TimeContext,
    /// Miniquad context access only for the crate
    pub(crate) quad_ctx: miniquad::Context,

    /// Controls whether or not the event loop should be running.
    /// Set this with `ggez::event::quit()`.
    pub continuing: bool,
}

impl Context {
    pub(crate) fn new(mut quad_ctx: miniquad::Context, conf: Conf) -> Context {
        let input_handler = Rc::new(RefCell::new(InputHandler::new()));

        Context {
            filesystem: Filesystem::new(&conf),
            gfx_context: graphics::GraphicsContext::new(&mut quad_ctx),
            audio_context: audio::AudioContext::new(),
            mouse_context: MouseContext::new(input_handler.clone()),
            keyboard_context: KeyboardContext::new(input_handler.clone()),
            timer_context: TimeContext::new(),
            quad_ctx,
            continuing: true,
        }
    }

    pub(crate) fn framebuffer(&mut self) -> Option<miniquad::RenderPass> {
        self.gfx_context
            .canvas
            .as_ref()
            .map(|canvas| canvas.offscreen_pass.clone())
    }
}
