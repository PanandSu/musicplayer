use anyhow::{Context, Result};
use cpal::traits::HostTrait;
use rodio::stream::{DeviceSinkBuilder, DeviceSinkError, MixerDeviceSink};
use rodio::Player;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum AudioNotification {
    #[allow(dead_code)]
    DeviceChanged(String),
    StreamError(String),
}

pub struct AudioEngine {
    sink: MixerDeviceSink,
    player: Option<Player>,
    volume: f32,
    previous_volume: f32,
    muted: bool,
    event_rx: Receiver<AudioNotification>,
}

fn open_custom_sink(
    tx: Sender<AudioNotification>,
) -> Result<MixerDeviceSink, DeviceSinkError> {
    let callback = move |err: cpal::Error| {
        let err_str = err.to_string();
        if err_str.to_lowercase().contains("device changed") {
            let _ = tx.send(AudioNotification::DeviceChanged(err_str));
        } else {
            let _ = tx.send(AudioNotification::StreamError(err_str));
        }
    };

    DeviceSinkBuilder::from_default_device()
        .and_then(|builder| builder.with_error_callback(callback.clone()).open_sink_or_fallback())
        .or_else(|orig_err| {
            if let Ok(devices) = cpal::default_host().output_devices() {
                for dev in devices {
                    if let Ok(builder) = DeviceSinkBuilder::from_device(dev) {
                        if let Ok(sink) = builder
                            .with_error_callback(callback.clone())
                            .open_sink_or_fallback()
                        {
                            return Ok(sink);
                        }
                    }
                }
            }
            Err(orig_err)
        })
}

impl AudioEngine {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let mut sink = open_custom_sink(tx)
            .map_err(|e| anyhow::anyhow!("Failed to open audio device: {:?}", e))
            .context("Failed to initialize audio output device")?;
        sink.log_on_drop(false);

        Ok(Self {
            sink,
            player: None,
            volume: 0.8,
            previous_volume: 0.8,
            muted: false,
            event_rx: rx,
        })
    }

    pub fn poll_notification(&self) -> Option<AudioNotification> {
        self.event_rx.try_recv().ok()
    }

    pub fn play_file(&mut self, path: &Path) -> Result<()> {
        if let Some(ref p) = self.player {
            p.stop();
        }

        let file = File::open(path)
            .with_context(|| format!("Failed to open audio file: {}", path.display()))?;
        let reader = BufReader::new(file);

        let player = rodio::stream::play(self.sink.mixer(), reader)
            .with_context(|| format!("Failed to decode and play audio file: {}", path.display()))?;

        player.set_volume(if self.muted { 0.0 } else { self.volume });
        self.player = Some(player);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn pause(&self) {
        if let Some(ref p) = self.player {
            p.pause();
        }
    }

    #[allow(dead_code)]
    pub fn resume(&self) {
        if let Some(ref p) = self.player {
            p.play();
        }
    }

    pub fn toggle_pause(&self) {
        if let Some(ref p) = self.player {
            if p.is_paused() {
                p.play();
            } else {
                p.pause();
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(ref p) = self.player {
            p.stop();
        }
        self.player = None;
    }

    pub fn is_paused(&self) -> bool {
        self.player.as_ref().map_or(false, |p| p.is_paused())
    }

    pub fn is_playing(&self) -> bool {
        self.player
            .as_ref()
            .map_or(false, |p| !p.is_paused() && !p.empty())
    }

    pub fn is_finished(&self) -> bool {
        self.player.as_ref().map_or(false, |p| p.empty())
    }

    pub fn get_position(&self) -> Duration {
        self.player
            .as_ref()
            .map_or(Duration::ZERO, |p| p.get_pos())
    }

    #[allow(dead_code)]
    pub fn seek_to(&self, pos: Duration) {
        if let Some(ref p) = self.player {
            let _ = p.try_seek(pos);
        }
    }

    pub fn seek_relative(&self, offset_secs: i64) {
        if let Some(ref p) = self.player {
            let current = p.get_pos();
            let new_secs = (current.as_secs_f64() + offset_secs as f64).max(0.0);
            let target = Duration::from_secs_f64(new_secs);
            let _ = p.try_seek(target);
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.5);
        if !self.muted {
            if let Some(ref p) = self.player {
                p.set_volume(self.volume);
            }
        }
    }

    pub fn volume_up(&mut self, step: f32) {
        if self.muted {
            self.muted = false;
        }
        self.set_volume(self.volume + step);
    }

    pub fn volume_down(&mut self, step: f32) {
        if self.muted {
            self.muted = false;
        }
        self.set_volume(self.volume - step);
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if self.muted {
            self.previous_volume = self.volume;
            if let Some(ref p) = self.player {
                p.set_volume(0.0);
            }
        } else {
            if let Some(ref p) = self.player {
                p.set_volume(self.volume);
            }
        }
    }
}
