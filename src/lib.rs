pub mod audio;
pub mod conf;
pub mod error;
pub mod event;
pub mod filesystem;
pub mod goodies;
pub mod graphics;
pub mod input;
pub mod timer;

mod context;

pub use crate::{
    context::Context, error::GameError, error::GameResult, event::EventHandler,
    goodies::matrix_transform_2d,
};
pub use cgmath;

#[cfg(feature = "log-impl")]
pub use miniquad::{debug, info, log, warn};

pub mod rand {
    use miniquad::rand;

    pub trait RandomRange {
        fn gen_range(low: Self, high: Self) -> Self;
    }

    impl RandomRange for f32 {
        fn gen_range(low: Self, high: Self) -> Self {
            let r = unsafe { rand() } as f32 / miniquad::RAND_MAX as f32;
            low + (high - low) * r
        }
    }
    impl RandomRange for i32 {
        fn gen_range(low: i32, high: i32) -> Self {
            let r = unsafe { rand() } as f32 / miniquad::RAND_MAX as f32;
            let r = low as f32 + (high as f32 - low as f32) * r;
            r as i32
        }
    }
    impl RandomRange for i16 {
        fn gen_range(low: i16, high: i16) -> Self {
            let r = unsafe { rand() } as f32 / miniquad::RAND_MAX as f32;
            let r = low as f32 + (high as f32 - low as f32) * r;
            r as i16
        }
    }

    impl RandomRange for usize {
        fn gen_range(low: usize, high: usize) -> Self {
            let r = unsafe { rand() } as f32 / miniquad::RAND_MAX as f32;
            let r = low as f32 + (high as f32 - low as f32) * r;
            r as usize
        }
    }

    pub fn gen_range<T>(low: T, high: T) -> T
    where
        T: RandomRange,
    {
        T::gen_range(low, high)
    }

    pub trait ChooseRandom<T> {
        fn choose(&mut self) -> Option<&mut T>;
    }

    impl<T> ChooseRandom<T> for Vec<T> {
        fn choose(&mut self) -> Option<&mut T> {
            let ix = gen_range(0, self.len());
            self.get_mut(ix)
        }
    }
}

struct EventHandlerWrapper {
    event_handler: Box<dyn event::EventHandler>,
    context: Context,
}

impl miniquad::EventHandlerFree for EventHandlerWrapper {
    fn update(&mut self) {
        if !self.context.continuing {
            self.context.quad_ctx.quit();
        }

        self.event_handler.update(&mut self.context).unwrap();
        if let Some(ref mut mixer) = &mut *self.context.audio_context.mixer.borrow_mut() {
            mixer.frame();
        }
        self.context.timer_context.tick();
    }

    fn draw(&mut self) {
        self.event_handler.draw(&mut self.context).unwrap();
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        self.event_handler
            .resize_event(&mut self.context, width, height);
    }

    fn key_down_event(
        &mut self,
        keycode: miniquad::KeyCode,
        _keymods: miniquad::KeyMods,
        repeat: bool,
    ) {
        self.event_handler.key_down_event(
            &mut self.context,
            keycode.into(),
            crate::input::keyboard::KeyMods::NONE,
            repeat,
        );
    }

    fn key_up_event(&mut self, keycode: miniquad::KeyCode, _keymods: miniquad::KeyMods) {
        self.event_handler.key_up_event(
            &mut self.context,
            keycode.into(),
            crate::input::keyboard::KeyMods::NONE,
        );
    }

    fn mouse_button_down_event(&mut self, button: miniquad::MouseButton, x: f32, y: f32) {
        self.event_handler
            .mouse_button_down_event(&mut self.context, button.into(), x, y);
    }

    fn mouse_button_up_event(&mut self, button: miniquad::MouseButton, x: f32, y: f32) {
        self.event_handler
            .mouse_button_up_event(&mut self.context, button.into(), x, y);
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.event_handler
            .mouse_motion_event(&mut self.context, x, y, 0., 0.);
    }

    fn touch_event(&mut self, phase: miniquad::TouchPhase, id: u64, x: f32, y: f32) {
        self.event_handler
            .touch_event(&mut self.context, phase, id, x, y);
    }
}

pub fn start<F>(conf: conf::Conf, f: F) -> GameResult
where
    F: 'static + FnOnce(&mut Context) -> Box<dyn EventHandler>,
{
    miniquad::start(
        miniquad::conf::Conf {
            window_height: conf.window_height,
            window_width: conf.window_width,
            fullscreen: conf.fullscreen,
            ..miniquad::conf::Conf::default()
        },
        |ctx| {
            let mut context = Context::new(ctx, conf);

            let (w, h) = context.quad_ctx.screen_size();
            context
                .gfx_context
                .set_screen_coordinates(graphics::Rect::new(0., 0., w as f32, h as f32));

            let event_handler = f(&mut context);

            miniquad::UserData::free(EventHandlerWrapper {
                event_handler,
                context,
            })
        },
    );
    Ok(())
}
