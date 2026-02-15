use bevy::prelude::*;
use std::sync::Arc;

/// Holds pre-generated sound effect asset handles.
#[derive(Resource)]
pub struct SoundEffects {
    pub eat: Handle<AudioSource>,
    pub death: Handle<AudioSource>,
    pub shrink_warning: Handle<AudioSource>,
    pub shrink_impact: Handle<AudioSource>,
    pub speed_up: Handle<AudioSource>,
    pub countdown_beep: Handle<AudioSource>,
    pub countdown_go: Handle<AudioSource>,
    pub game_over: Handle<AudioSource>,
}

const SAMPLE_RATE: u32 = 22050;

/// Generate a WAV file (mono, 16-bit PCM) from raw samples in [-1.0, 1.0].
fn samples_to_wav(samples: &[f32]) -> Vec<u8> {
    let num_samples = samples.len();
    let data_size = (num_samples * 2) as u32; // 16-bit = 2 bytes per sample
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(file_size as usize + 8);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes()); // sample rate
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // byte rate
    wav.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let val = (clamped * 32767.0) as i16;
        wav.extend_from_slice(&val.to_le_bytes());
    }

    wav
}

/// Short rising bleep for eating food (~80ms)
fn gen_eat() -> Vec<f32> {
    let duration = 0.08;
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 440.0 + (880.0 - 440.0) * (t / duration);
        let envelope = 1.0 - (t / duration); // linear decay
        let val = (2.0 * std::f32::consts::PI * freq * t).sin() * envelope * 0.5;
        samples.push(val);
    }
    samples
}

/// Harsh noise burst + low tone for death (~200ms)
fn gen_death() -> Vec<f32> {
    let duration = 0.2;
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(n);
    let mut noise_state: u32 = 0xDEAD_BEEF;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let envelope = (1.0 - t / duration).powi(2);
        // Low rumble
        let low = (2.0 * std::f32::consts::PI * 60.0 * t).sin() * 0.4;
        // Noise
        noise_state = noise_state.wrapping_mul(1103515245).wrapping_add(12345);
        let noise = ((noise_state >> 16) as f32 / 32768.0 - 1.0) * 0.6;
        samples.push((low + noise) * envelope * 0.5);
    }
    samples
}

/// Alternating alarm tone for shrink warning (~500ms)
fn gen_shrink_warning() -> Vec<f32> {
    let duration = 0.5;
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        // Alternate between 600Hz and 800Hz every 0.1s
        let freq = if ((t * 10.0) as u32) % 2 == 0 { 600.0 } else { 800.0 };
        let envelope = 0.8 - 0.3 * (t / duration);
        let val = (2.0 * std::f32::consts::PI * freq * t).sin() * envelope * 0.35;
        samples.push(val);
    }
    samples
}

/// Deep thud for arena shrink impact (~300ms)
fn gen_shrink_impact() -> Vec<f32> {
    let duration = 0.3;
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(n);
    let mut noise_state: u32 = 0xCAFE_BABE;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let envelope = (1.0 - t / duration).powi(3);
        let low = (2.0 * std::f32::consts::PI * 80.0 * t).sin() * 0.7;
        noise_state = noise_state.wrapping_mul(1103515245).wrapping_add(12345);
        let noise = ((noise_state >> 16) as f32 / 32768.0 - 1.0) * 0.3;
        samples.push((low + noise) * envelope * 0.5);
    }
    samples
}

/// Ascending frequency sweep for speed increase (~300ms)
fn gen_speed_up() -> Vec<f32> {
    let duration = 0.3;
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 200.0 + 1000.0 * (t / duration).powi(2);
        let envelope = (1.0 - (t / duration).powi(2)) * 0.8;
        let val = (2.0 * std::f32::consts::PI * freq * t).sin() * envelope * 0.4;
        samples.push(val);
    }
    samples
}

/// Short beep for countdown ticks (~100ms)
fn gen_countdown_beep() -> Vec<f32> {
    let duration = 0.1;
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let envelope = 1.0 - (t / duration);
        let val = (2.0 * std::f32::consts::PI * 880.0 * t).sin() * envelope * 0.4;
        samples.push(val);
    }
    samples
}

/// Bright chord burst for "GO!" (~200ms)
fn gen_countdown_go() -> Vec<f32> {
    let duration = 0.2;
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let envelope = (1.0 - t / duration).sqrt();
        // Major chord: root + major third + fifth
        let root = (2.0 * std::f32::consts::PI * 523.25 * t).sin(); // C5
        let third = (2.0 * std::f32::consts::PI * 659.25 * t).sin(); // E5
        let fifth = (2.0 * std::f32::consts::PI * 783.99 * t).sin(); // G5
        let val = (root + third * 0.8 + fifth * 0.6) / 2.4 * envelope * 0.5;
        samples.push(val);
    }
    samples
}

/// Descending tone for game over (~500ms)
fn gen_game_over() -> Vec<f32> {
    let duration = 0.5;
    let n = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 800.0 - 600.0 * (t / duration);
        let envelope = (1.0 - t / duration).sqrt();
        // Add a slight wobble for sadness
        let wobble = 1.0 + 0.02 * (2.0 * std::f32::consts::PI * 6.0 * t).sin();
        let val = (2.0 * std::f32::consts::PI * freq * wobble * t).sin() * envelope * 0.45;
        samples.push(val);
    }
    samples
}

fn make_audio_source(samples: Vec<f32>) -> AudioSource {
    let wav = samples_to_wav(&samples);
    AudioSource {
        bytes: Arc::from(wav.into_boxed_slice()),
    }
}

/// Startup system: generate all sound effects and store handles in a resource.
pub fn setup_audio(mut commands: Commands, mut audio_assets: ResMut<Assets<AudioSource>>) {
    let sfx = SoundEffects {
        eat: audio_assets.add(make_audio_source(gen_eat())),
        death: audio_assets.add(make_audio_source(gen_death())),
        shrink_warning: audio_assets.add(make_audio_source(gen_shrink_warning())),
        shrink_impact: audio_assets.add(make_audio_source(gen_shrink_impact())),
        speed_up: audio_assets.add(make_audio_source(gen_speed_up())),
        countdown_beep: audio_assets.add(make_audio_source(gen_countdown_beep())),
        countdown_go: audio_assets.add(make_audio_source(gen_countdown_go())),
        game_over: audio_assets.add(make_audio_source(gen_game_over())),
    };
    commands.insert_resource(sfx);
}

/// Play a one-shot sound by spawning an AudioPlayer entity that despawns when done.
pub fn play_sfx(commands: &mut Commands, handle: &Handle<AudioSource>) {
    commands.spawn((
        AudioPlayer::new(handle.clone()),
        PlaybackSettings::DESPAWN,
    ));
}
