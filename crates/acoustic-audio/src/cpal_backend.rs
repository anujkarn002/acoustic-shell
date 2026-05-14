use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, Stream, StreamConfig,
};
use crossbeam_channel::{unbounded, Sender};
use acoustic_core::note::Note;
use crate::{port::AudioPort, voice::Voice};

enum Cmd {
    NoteOn  { midi: u8, freq: f32, velocity: u8 },
    NoteOff { midi: u8 },
    Volume(f32),
}

/// Audio backend powered by cpal. Sends commands to the audio callback via a
/// lock-free channel — never blocks the audio thread.
pub struct CpalAudioBackend {
    _stream: Stream, // kept alive; dropping it stops audio
    cmd_tx: Sender<Cmd>,
}

impl CpalAudioBackend {
    pub fn new() -> anyhow::Result<Self> {
        let host   = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("no audio output device found"))?;

        let supported = device.default_output_config()?;
        let channels    = supported.channels() as usize;
        let sample_rate = supported.sample_rate().0 as f32;

        let config = StreamConfig {
            channels:    supported.channels(),
            sample_rate: supported.sample_rate(),
            // 256 frames @ 44100 Hz ≈ 5.8ms app-side latency.
            // PulseAudio/WSLg adds its own buffer (~40-80ms) on top; keeping
            // our buffer small minimises the part we can control.
            buffer_size: cpal::BufferSize::Fixed(256),
        };

        let (cmd_tx, cmd_rx) = unbounded::<Cmd>();

        let stream = match supported.sample_format() {
            SampleFormat::F32 => build_stream::<f32>(
                &device, &config, cmd_rx, sample_rate, channels,
            )?,
            SampleFormat::I16 => build_stream::<i16>(
                &device, &config, cmd_rx, sample_rate, channels,
            )?,
            SampleFormat::U16 => build_stream::<u16>(
                &device, &config, cmd_rx, sample_rate, channels,
            )?,
            fmt => anyhow::bail!("unsupported sample format: {fmt:?}"),
        };

        stream.play()?;
        Ok(Self { _stream: stream, cmd_tx })
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    cmd_rx: crossbeam_channel::Receiver<Cmd>,
    sample_rate: f32,
    channels: usize,
) -> anyhow::Result<Stream>
where
    T: cpal::SizedSample + cpal::FromSample<f32> + Send + 'static,
{
    let mut voices: Vec<Voice> = Vec::with_capacity(16);
    let mut volume = 0.7f32;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            // Drain commands — lock-free, never blocks
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    Cmd::NoteOn { midi, freq, velocity } => {
                        voices.retain(|v| v.midi != midi); // stop any existing voice on this key
                        if voices.len() < 16 {
                            voices.push(Voice::new(midi, freq, velocity, sample_rate));
                        }
                    }
                    Cmd::NoteOff { midi } => {
                        for v in voices.iter_mut().filter(|v| v.midi == midi) {
                            v.release();
                        }
                    }
                    Cmd::Volume(v) => volume = v.clamp(0.0, 1.0),
                }
            }

            // Fill buffer
            for frame in data.chunks_mut(channels) {
                let mixed: f32 = voices.iter_mut().map(|v| v.next_sample()).sum();
                // Soft-clip via tanh; 0.3 headroom for up to ~10 simultaneous voices
                let out = T::from_sample_((mixed * 0.3 * volume).tanh());
                for s in frame.iter_mut() {
                    *s = out;
                }
            }

            voices.retain(|v| v.is_active());
        },
        |err| eprintln!("audio stream error: {err}"),
        None,
    )?;

    Ok(stream)
}

impl AudioPort for CpalAudioBackend {
    fn play_note(&mut self, note: Note, velocity: u8) {
        let _ = self.cmd_tx.send(Cmd::NoteOn {
            midi: note.midi_number(),
            freq: note.frequency_hz(),
            velocity,
        });
    }

    fn stop_note(&mut self, note: Note) {
        let _ = self.cmd_tx.send(Cmd::NoteOff { midi: note.midi_number() });
    }

    fn set_volume(&mut self, volume: f32) {
        let _ = self.cmd_tx.send(Cmd::Volume(volume));
    }
}
