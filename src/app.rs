use crate::audio::AudioEngine;
use crate::file_browser::FileBrowser;
use crate::playlist::Playlist;
use crate::visualizer::SpectrumVisualizer;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    FileBrowser,
    Playlist,
}

pub struct App {
    pub audio: AudioEngine,
    pub playlist: Playlist,
    pub browser: FileBrowser,
    pub visualizer: SpectrumVisualizer,
    pub active_panel: ActivePanel,
    pub show_help: bool,
    pub is_searching: bool,
    pub search_query: String,
    pub status_message: Option<(String, Instant)>,
    pub should_quit: bool,
    pub needs_clear: bool,
    pub browser_state: ListState,
    pub playlist_state: ListState,
    auto_advance_debounce: Instant,
}

impl App {
    pub fn new() -> Result<Self> {
        let audio = AudioEngine::new()?;
        let playlist = Playlist::new();
        let browser = FileBrowser::new(None);
        let visualizer = SpectrumVisualizer::new();

        let mut app = Self {
            audio,
            playlist,
            browser,
            visualizer,
            active_panel: ActivePanel::FileBrowser,
            show_help: false,
            is_searching: false,
            search_query: String::new(),
            status_message: None,
            should_quit: false,
            needs_clear: false,
            browser_state: ListState::default(),
            playlist_state: ListState::default(),
            auto_advance_debounce: Instant::now(),
        };

        // Automatically populate playlist with audio files in current directory
        let local_audio = app.browser.collect_all_audio_in_dir();
        if !local_audio.is_empty() {
            app.playlist.add_files(&local_audio);
            app.set_status(format!(
                "Loaded {} audio tracks from current directory",
                local_audio.len()
            ));
        }

        Ok(app)
    }

    pub fn set_status<S: Into<String>>(&mut self, msg: S) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    pub fn current_status(&self) -> Option<&str> {
        if let Some((ref msg, time)) = self.status_message {
            if time.elapsed() < Duration::from_secs(4) {
                return Some(msg.as_str());
            }
        }
        None
    }

    pub fn on_tick(&mut self) {
        while let Some(notification) = self.audio.poll_notification() {
            match notification {
                crate::audio::AudioNotification::DeviceChanged(_) => {
                    self.set_status("🎧 Audio output switched to new device");
                    self.needs_clear = true;
                }
                crate::audio::AudioNotification::StreamError(err) => {
                    self.set_status(format!("Audio alert: {}", err));
                    self.needs_clear = true;
                }
            }
        }

        let is_playing = self.audio.is_playing();
        self.visualizer.update(is_playing);

        // Check if track has finished playing
        if self.playlist.current_track().is_some()
            && self.audio.is_finished()
            && self.auto_advance_debounce.elapsed() > Duration::from_millis(500)
        {
            self.auto_advance_debounce = Instant::now();
            self.play_next();
        }
    }

    pub fn play_track_at_index(&mut self, index: usize) {
        if let Some(track) = self.playlist.set_current_index(index).cloned() {
            let path = track.metadata.path.clone();
            let title = track.metadata.display_title().to_string();
            match self.audio.play_file(&path) {
                Ok(()) => {
                    self.set_status(format!("Now playing: {}", title));
                }
                Err(e) => {
                    self.set_status(format!("Playback error: {}", e));
                }
            }
        }
    }

    pub fn play_selected_playlist_track(&mut self) {
        let selected = self.playlist.selected_index();
        self.play_track_at_index(selected);
    }

    pub fn play_next(&mut self) {
        if let Some(next_idx) = self.playlist.next_track_index() {
            self.play_track_at_index(next_idx);
        } else {
            self.audio.stop();
            self.set_status("End of playlist");
        }
    }

    pub fn play_prev(&mut self) {
        if let Some(prev_idx) = self.playlist.prev_track_index() {
            self.play_track_at_index(prev_idx);
        }
    }

    pub fn add_path_to_playlist(&mut self, path: &Path, start_playing: bool) {
        self.playlist.add_file(path);
        let added_idx = self.playlist.len() - 1;
        if start_playing || self.playlist.current_track().is_none() {
            self.play_track_at_index(added_idx);
        } else {
            let file_name = path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "track".to_string());
            self.set_status(format!("Added to playlist: {}", file_name));
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') || key.code == KeyCode::Char('q') {
                self.show_help = false;
            }
            return;
        }

        if self.is_searching {
            match key.code {
                KeyCode::Esc => {
                    self.is_searching = false;
                    self.search_query.clear();
                    self.browser.clear_filter();
                }
                KeyCode::Enter => {
                    self.is_searching = false;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.browser.set_filter(self.search_query.clone());
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.browser.set_filter(self.search_query.clone());
                }
                _ => {}
            }
            return;
        }

        // Global keybindings
        if key.code == KeyCode::Char('l') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.needs_clear = true;
            self.set_status("Screen redrawn");
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
                return;
            }
            KeyCode::Tab => {
                self.active_panel = match self.active_panel {
                    ActivePanel::FileBrowser => ActivePanel::Playlist,
                    ActivePanel::Playlist => ActivePanel::FileBrowser,
                };
                return;
            }
            KeyCode::Char(' ') => {
                if self.playlist.current_track().is_none() && !self.playlist.is_empty() {
                    self.play_track_at_index(0);
                } else {
                    self.audio.toggle_pause();
                    if self.audio.is_paused() {
                        self.set_status("Paused");
                    } else {
                        self.set_status("Playing");
                    }
                }
                return;
            }
            KeyCode::Char('n') => {
                self.play_next();
                return;
            }
            KeyCode::Char('p') => {
                self.play_prev();
                return;
            }
            KeyCode::Char('s') => {
                self.playlist.toggle_shuffle();
                let status = if self.playlist.is_shuffle() {
                    "Shuffle: ON"
                } else {
                    "Shuffle: OFF"
                };
                self.set_status(status);
                return;
            }
            KeyCode::Char('r') => {
                self.playlist.cycle_repeat();
                self.set_status(format!("Repeat: {}", self.playlist.repeat_mode().display_label()));
                return;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.audio.volume_up(0.05);
                self.set_status(format!("Volume: {:.0}%", self.audio.volume() * 100.0));
                return;
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                self.audio.volume_down(0.05);
                self.set_status(format!("Volume: {:.0}%", self.audio.volume() * 100.0));
                return;
            }
            KeyCode::Char('m') => {
                self.audio.toggle_mute();
                if self.audio.is_muted() {
                    self.set_status("Muted");
                } else {
                    self.set_status(format!("Volume: {:.0}%", self.audio.volume() * 100.0));
                }
                return;
            }
            KeyCode::Left => {
                self.audio.seek_relative(-5);
                self.set_status("Seek -5s");
                return;
            }
            KeyCode::Right => {
                self.audio.seek_relative(5);
                self.set_status("Seek +5s");
                return;
            }
            _ => {}
        }

        // Panel specific keybindings
        match self.active_panel {
            ActivePanel::FileBrowser => self.handle_browser_key(key),
            ActivePanel::Playlist => self.handle_playlist_key(key),
        }
    }

    fn handle_browser_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.browser.select_next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.browser.select_prev();
            }
            KeyCode::PageDown => {
                self.browser.select_page_down(10);
            }
            KeyCode::PageUp => {
                self.browser.select_page_up(10);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.browser.select_first();
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.browser.select_last();
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.browser.select_page_down(10);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.browser.select_page_up(10);
            }
            KeyCode::Char('h') | KeyCode::Backspace => {
                self.browser.go_up();
            }
            KeyCode::Char('/') => {
                self.is_searching = true;
                self.search_query.clear();
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(audio_path) = self.browser.activate_selected() {
                    self.add_path_to_playlist(&audio_path, true);
                    self.active_panel = ActivePanel::Playlist;
                }
            }
            KeyCode::Char('a') => {
                if let Some(item) = self.browser.selected_item().cloned() {
                    if let crate::file_browser::ItemType::AudioFile(_) = item.item_type {
                        self.add_path_to_playlist(&item.path, false);
                    }
                }
            }
            KeyCode::Char('A') => {
                let all = self.browser.collect_all_audio_in_dir();
                let count = all.len();
                self.playlist.add_files(&all);
                self.set_status(format!("Added {} tracks to playlist", count));
            }
            _ => {}
        }
    }

    fn handle_playlist_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.playlist.select_next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.playlist.select_prev();
            }
            KeyCode::PageDown => {
                self.playlist.select_page_down(10);
            }
            KeyCode::PageUp => {
                self.playlist.select_page_up(10);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.playlist.select_first();
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.playlist.select_last();
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.playlist.select_page_down(10);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.playlist.select_page_up(10);
            }
            KeyCode::Enter => {
                self.play_selected_playlist_track();
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                let sel = self.playlist.selected_index();
                if let Some(track) = self.playlist.remove_at(sel) {
                    self.set_status(format!("Removed: {}", track.metadata.display_title()));
                }
            }
            KeyCode::Char('c') => {
                self.playlist.clear();
                self.audio.stop();
                self.set_status("Playlist cleared");
            }
            _ => {}
        }
    }
}
