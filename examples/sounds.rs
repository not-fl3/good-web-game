extern crate good_web_game as ggez;
extern crate nalgebra as na;

use ggez::audio;
use ggez::event;
use ggez::graphics;
use ggez::input;
use ggez::{Context, GameResult};

use std::time::Duration;

struct MainState {
    sound: audio::SoundSource,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let sound = audio::SoundSource::new(ctx, "/sound.ogg")?;
        let s = MainState { sound };
        Ok(s)
    }

    // To test: play, play_later, play_detached(),
    // set_repeat, set_fade_in, set_pitch,
    // basically every method on SoundSource, actually,
    // then the same ones for `SpatialSoundSource`.

    /// Plays the sound multiple times
    fn play_detached(&mut self, _ctx: &mut Context) {
        // "detached" sounds keep playing even after they are dropped
        let _ = self.sound.play_detached();
    }

    /// Waits until the sound is done playing before playing again.
    fn play_later(&mut self, _ctx: &mut Context) {
        let _ = self.sound.play_later();
    }

    /// Fades the sound in over a second
    /// Which isn't really ideal 'cause the sound is barely a second long, but still.
    fn play_fadein(&mut self, ctx: &mut Context) {
        let mut sound = audio::SoundSource::new(ctx, "/sound.ogg").unwrap();
        sound.set_fade_in(Duration::from_millis(1000));
        sound.play_detached().unwrap();
    }

    fn play_highpitch(&mut self, ctx: &mut Context) {
        let mut sound = audio::SoundSource::new(ctx, "/sound.ogg").unwrap();
        sound.set_pitch(2.0);
        sound.play_detached().unwrap();
    }
    fn play_lowpitch(&mut self, ctx: &mut Context) {
        let mut sound = audio::SoundSource::new(ctx, "/sound.ogg").unwrap();
        sound.set_pitch(0.5);
        sound.play_detached().unwrap();
    }

    /// Plays the sound and prints out stats until it's done.
    fn play_stats(&mut self, _ctx: &mut Context) {
        self.sound.play().unwrap();
        while self.sound.playing() {
            println!("Elapsed time: {:?}", self.sound.elapsed())
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let text = graphics::Text::new((
            "Press number keys 1-6 to play a sound, or escape to quit.",
            30.,
        ));
        graphics::draw(ctx, &text, (na::Point2::new(100.0, 100.), graphics::WHITE))?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        _button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        audio::maybe_create_soundmixer(ctx);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: event::KeyCode,
        _keymod: input::keyboard::KeyMods,
        _repeat: bool,
    ) {
        audio::maybe_create_soundmixer(ctx);

        match keycode {
            event::KeyCode::Key1 => self.play_detached(ctx),
            event::KeyCode::Key2 => self.play_later(ctx),
            event::KeyCode::Key3 => self.play_fadein(ctx),
            event::KeyCode::Key4 => self.play_highpitch(ctx),
            event::KeyCode::Key5 => self.play_lowpitch(ctx),
            event::KeyCode::Key6 => self.play_stats(ctx),
            event::KeyCode::Escape => {}
            _ => (),
        }
    }
}

pub fn main() -> GameResult {
    ggez::start(
        ggez::conf::Conf {
            cache: ggez::conf::Cache::Tar(include_bytes!("resources.tar").to_vec()),
            loading: ggez::conf::Loading::Embedded,
            ..Default::default()
        },
        |ctx| Box::new(MainState::new(ctx).unwrap()),
    )
}
