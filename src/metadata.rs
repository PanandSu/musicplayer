use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "wav", "ogg", "m4a", "aac", "alac", "aiff", "aif", "opus", "caf",
];

#[derive(Debug, Clone)]
pub struct SongMetadata {
    pub path: PathBuf,
    pub file_name: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Duration,
    pub sample_rate: Option<u32>,
    pub bitrate: Option<u32>,
    pub channels: Option<u8>,
    pub format: String,
}

impl SongMetadata {
    pub fn display_title(&self) -> &str {
        if !self.title.is_empty() {
            &self.title
        } else {
            &self.file_name
        }
    }

    pub fn display_artist(&self) -> &str {
        if !self.artist.is_empty() {
            &self.artist
        } else {
            "Unknown Artist"
        }
    }

    pub fn display_album(&self) -> &str {
        if !self.album.is_empty() {
            &self.album
        } else {
            "Unknown Album"
        }
    }
}

pub fn is_supported_audio_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str())
        })
        .unwrap_or(false)
}

pub fn extract_metadata(path: &Path) -> SongMetadata {
    let file_name = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let fallback_title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&file_name)
        .to_string();

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_uppercase())
        .unwrap_or_else(|| "AUDIO".to_string());

    let mut metadata = SongMetadata {
        path: path.to_path_buf(),
        file_name,
        title: fallback_title,
        artist: String::new(),
        album: String::new(),
        duration: Duration::ZERO,
        sample_rate: None,
        bitrate: None,
        channels: None,
        format: ext,
    };

    if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read().map_err(Into::into)) {
        let properties = tagged_file.properties();
        metadata.duration = properties.duration();
        metadata.sample_rate = properties.sample_rate();
        metadata.bitrate = properties.audio_bitrate();
        metadata.channels = properties.channels();

        if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
            if let Some(title) = tag.title() {
                let trimmed = title.trim();
                if !trimmed.is_empty() {
                    metadata.title = trimmed.to_string();
                }
            }
            if let Some(artist) = tag.artist() {
                let trimmed = artist.trim();
                if !trimmed.is_empty() {
                    metadata.artist = trimmed.to_string();
                }
            }
            if let Some(album) = tag.album() {
                let trimmed = album.trim();
                if !trimmed.is_empty() {
                    metadata.album = trimmed.to_string();
                }
            }
        }
    }

    metadata
}

pub fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}
