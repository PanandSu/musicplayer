use crate::metadata::{extract_metadata, SongMetadata};
use rand::seq::SliceRandom;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

impl RepeatMode {
    pub fn cycle(&self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            RepeatMode::Off => "Off",
            RepeatMode::All => "All",
            RepeatMode::One => "Track",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Track {
    #[allow(dead_code)]
    pub id: usize,
    pub metadata: SongMetadata,
}

pub struct Playlist {
    tracks: Vec<Track>,
    next_id: usize,
    current_idx: Option<usize>,
    selected_idx: usize,
    repeat_mode: RepeatMode,
    shuffle: bool,
    shuffled_order: Vec<usize>,
    shuffle_pos: usize,
}

impl Playlist {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            next_id: 1,
            current_idx: None,
            selected_idx: 0,
            repeat_mode: RepeatMode::All,
            shuffle: false,
            shuffled_order: Vec::new(),
            shuffle_pos: 0,
        }
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_idx
    }

    pub fn selected_index(&self) -> usize {
        self.selected_idx
    }

    #[allow(dead_code)]
    pub fn set_selected_index(&mut self, idx: usize) {
        if !self.tracks.is_empty() {
            self.selected_idx = idx.min(self.tracks.len() - 1);
        }
    }

    pub fn select_next(&mut self) {
        if !self.tracks.is_empty() && self.selected_idx + 1 < self.tracks.len() {
            self.selected_idx += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
    }

    pub fn select_page_down(&mut self, step: usize) {
        if !self.tracks.is_empty() {
            self.selected_idx = (self.selected_idx + step).min(self.tracks.len() - 1);
        }
    }

    pub fn select_page_up(&mut self, step: usize) {
        self.selected_idx = self.selected_idx.saturating_sub(step);
    }

    pub fn select_first(&mut self) {
        self.selected_idx = 0;
    }

    pub fn select_last(&mut self) {
        if !self.tracks.is_empty() {
            self.selected_idx = self.tracks.len() - 1;
        }
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }

    pub fn cycle_repeat(&mut self) {
        self.repeat_mode = self.repeat_mode.cycle();
    }

    pub fn is_shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        if self.shuffle {
            self.rebuild_shuffle_order();
        }
    }

    fn rebuild_shuffle_order(&mut self) {
        let count = self.tracks.len();
        if count == 0 {
            self.shuffled_order.clear();
            self.shuffle_pos = 0;
            return;
        }

        let mut order: Vec<usize> = (0..count).collect();
        let mut rng = rand::rng();
        order.shuffle(&mut rng);

        // Put current track at the front of shuffle order if active
        if let Some(curr) = self.current_idx {
            if let Some(pos) = order.iter().position(|&x| x == curr) {
                order.swap(0, pos);
            }
        }

        self.shuffled_order = order;
        self.shuffle_pos = 0;
    }

    pub fn add_file(&mut self, path: &Path) {
        let metadata = extract_metadata(path);
        let track = Track {
            id: self.next_id,
            metadata,
        };
        self.next_id += 1;
        self.tracks.push(track);
        if self.shuffle {
            self.rebuild_shuffle_order();
        }
    }

    pub fn add_files(&mut self, paths: &[PathBuf]) {
        for path in paths {
            let metadata = extract_metadata(path);
            let track = Track {
                id: self.next_id,
                metadata,
            };
            self.next_id += 1;
            self.tracks.push(track);
        }
        if self.shuffle {
            self.rebuild_shuffle_order();
        }
    }

    pub fn remove_at(&mut self, index: usize) -> Option<Track> {
        if index >= self.tracks.len() {
            return None;
        }

        let removed = self.tracks.remove(index);

        if let Some(curr) = self.current_idx {
            if curr == index {
                if self.tracks.is_empty() {
                    self.current_idx = None;
                } else if curr >= self.tracks.len() {
                    self.current_idx = Some(self.tracks.len() - 1);
                }
            } else if curr > index {
                self.current_idx = Some(curr - 1);
            }
        }

        if !self.tracks.is_empty() && self.selected_idx >= self.tracks.len() {
            self.selected_idx = self.tracks.len() - 1;
        }

        if self.shuffle {
            self.rebuild_shuffle_order();
        }

        Some(removed)
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_idx = None;
        self.selected_idx = 0;
        self.shuffled_order.clear();
        self.shuffle_pos = 0;
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current_idx.and_then(|idx| self.tracks.get(idx))
    }

    pub fn set_current_index(&mut self, idx: usize) -> Option<&Track> {
        if idx < self.tracks.len() {
            self.current_idx = Some(idx);
            self.selected_idx = idx;
            if self.shuffle {
                if let Some(pos) = self.shuffled_order.iter().position(|&x| x == idx) {
                    self.shuffle_pos = pos;
                }
            }
            self.tracks.get(idx)
        } else {
            None
        }
    }

    pub fn next_track_index(&mut self) -> Option<usize> {
        if self.tracks.is_empty() {
            return None;
        }

        match self.repeat_mode {
            RepeatMode::One => self.current_idx,
            RepeatMode::All | RepeatMode::Off => {
                if self.shuffle {
                    if self.shuffled_order.is_empty() {
                        self.rebuild_shuffle_order();
                    }
                    if self.shuffle_pos + 1 < self.shuffled_order.len() {
                        self.shuffle_pos += 1;
                        self.shuffled_order.get(self.shuffle_pos).copied()
                    } else if self.repeat_mode == RepeatMode::All {
                        self.rebuild_shuffle_order();
                        self.shuffle_pos = 0;
                        self.shuffled_order.first().copied()
                    } else {
                        None
                    }
                } else {
                    match self.current_idx {
                        Some(idx) => {
                            if idx + 1 < self.tracks.len() {
                                Some(idx + 1)
                            } else if self.repeat_mode == RepeatMode::All {
                                Some(0)
                            } else {
                                None
                            }
                        }
                        None => Some(0),
                    }
                }
            }
        }
    }

    pub fn prev_track_index(&mut self) -> Option<usize> {
        if self.tracks.is_empty() {
            return None;
        }

        if self.shuffle {
            if self.shuffle_pos > 0 {
                self.shuffle_pos -= 1;
                self.shuffled_order.get(self.shuffle_pos).copied()
            } else if self.repeat_mode == RepeatMode::All && !self.shuffled_order.is_empty() {
                self.shuffle_pos = self.shuffled_order.len() - 1;
                self.shuffled_order.get(self.shuffle_pos).copied()
            } else {
                self.current_idx
            }
        } else {
            match self.current_idx {
                Some(idx) => {
                    if idx > 0 {
                        Some(idx - 1)
                    } else if self.repeat_mode == RepeatMode::All {
                        Some(self.tracks.len() - 1)
                    } else {
                        Some(0)
                    }
                }
                None => Some(0),
            }
        }
    }
}
