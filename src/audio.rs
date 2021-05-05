use crate::{error::GameError, filesystem, Context, GameResult};

use quad_snd::{
    decoder,
    mixer::{Sound, SoundMixer},
};

use std::cell::RefCell;
use std::rc::Rc;

pub struct AudioContext {
    pub(crate) mixer: Rc<RefCell<Option<SoundMixer>>>,
}

impl AudioContext {
    pub fn new() -> AudioContext {
        AudioContext {
            mixer: Rc::new(RefCell::new(None)),
        }
    }
}
pub struct SoundSource {
    sound: Sound,
    mixer: Rc<RefCell<Option<SoundMixer>>>,
}

impl SoundSource {
    pub fn new(ctx: &mut Context, path: &str) -> GameResult<SoundSource> {
        use std::io::Read;

        let mut file = filesystem::open(ctx, path)?;

        let mut bytes = vec![];
        file.bytes.read_to_end(&mut bytes)?;

        let sound;
        if path.ends_with(".ogg") {
            sound = decoder::read_ogg(&bytes).map_err(|_| GameError::SoundError)?;
        } else if path.ends_with(".wav") {
            sound = decoder::read_wav(&bytes).map_err(|_| GameError::SoundError)?;
        } else {
            panic!("Unsupported format. NOTE: gwg determines file format by path string, path should and with .wav or .ogg");
        };
        Ok(SoundSource {
            sound,
            mixer: ctx.audio_context.mixer.clone(),
        })
    }

    pub fn play(&mut self) -> GameResult<()> {
        if let Some(ref mut mixer) = &mut *self.mixer.borrow_mut() {
            mixer.play(self.sound.clone());

            Ok(())
        } else {
            Err(GameError::MixerNotCreated)
        }
    }

    pub fn playing(&self) -> bool {
        false
    }

    pub fn elapsed(&self) -> f32 {
        0.
    }

    /// Play source "in the background"; cannot be stopped
    /// anything blocking is impossible on wasm, so will display a warning and just play in background
    pub fn play_detached(&mut self) -> GameResult<()> {
        if let Some(ref mut mixer) = &mut *self.mixer.borrow_mut() {
            mixer.play(self.sound.clone());

            Ok(())
        } else {
            Err(GameError::MixerNotCreated)
        }
    }

    pub fn play_later(&mut self) {}

    pub fn set_fade_in(&mut self, _dur: std::time::Duration) {}

    pub fn set_pitch(&mut self, _ratio: f32) {}
}

/// This function should be called in one of the interaction event callbacks before any usages of audio
/// Because web.
pub fn maybe_create_soundmixer(ctx: &mut Context) {
    let mut mixer = ctx.audio_context.mixer.borrow_mut();
    if mixer.is_none() {
        *mixer = Some(SoundMixer::new());
    }
}
