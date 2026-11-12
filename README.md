# 🎵 Ratatui Music Player

A fast, terminal-based music player (TUI) written in Rust using [Ratatui](https://github.com/ratatui/ratatui), [Rodio](https://github.com/RustAudio/rodio) with Symphonia codecs, and [Lofty](https://github.com/Serial-ATA/lofty-rs).

Inspired by modern CLI/TUI players like `termusic`, `ratune`, `sparkplayer`, and `kew`.

---

## ✨ Features

- **Multi-Format Audio Support**:
  - FLAC, MP3, WAV, OGG/Vorbis, AAC, M4A, ALAC, AIFF, and Opus.
- **Dual-Panel TUI Layout**:
  - **File Explorer**: Browse directories, filter/search files, and inspect files with audio type badges (`[FLAC]`, `[MP3]`, etc.).
  - **Playlist Queue**: Track order, currently playing indicators, durations, and queue manipulation (add, remove, clear).
  - **Now Playing & Audio Info**: Track title, artist, album, audio format, sample rate (e.g. 44.1 kHz), and bitrate.
  - **Dynamic Spectrum Visualizer**: Animated harmonic frequency bars with peak hold and color gradient (Cyan -> Magenta -> Yellow).
  - **Progress Bar & Playback Stats**: Elapsed and remaining time, percentage, and seek capabilities.
- **Playback Modes**:
  - Repeat: `All`, `Track`, or `Off`.
  - Shuffle: Randomized playback sequence.
- **Vim-Friendly & Intuitive Keybindings**:
  - Seamless navigation and shortcuts with popup help (`?`).
- **Resilient & Safe Terminal Handling**:
  - Panic hook ensures terminal state is restored safely on unexpected exits.

---

## ⌨️ Controls & Keybindings

| Key | Action |
| :--- | :--- |
| `Space` | Play / Pause |
| `Tab` | Switch focus between **File Explorer** and **Playlist Queue** |
| `n` / `p` | Skip to **Next** / **Previous** track |
| `Left` / `Right` | Seek backward / forward 5 seconds |
| `+` / `-` | Volume Up / Down (5% steps) |
| `m` | Toggle Mute / Unmute |
| `s` | Toggle **Shuffle** mode (ON / OFF) |
| `r` | Cycle **Repeat** mode (Off -> All -> Track) |
| `?` | Toggle Help popup window |
| `q` / `Esc` | Quit player |

### File Explorer Controls
| Key | Action |
| :--- | :--- |
| `j` / `k` (or `↓` / `↑`) | Navigate files and folders |
| `PageDown` / `PageUp` | Scroll down / up by 10 items (`Ctrl+d` / `Ctrl+u`) |
| `Home` / `End` (`g` / `G`) | Jump to top / bottom of file list |
| `Enter` / `l` | Enter folder OR add & play selected audio |
| `h` / `Backspace` | Go up to parent folder |
| `a` | Add selected audio track to playlist queue (background) |
| `A` | Add all audio tracks in current folder to playlist |
| `/` | Search / filter files by name |

### Playlist Queue Controls
| Key | Action |
| :--- | :--- |
| `j` / `k` (or `↓` / `↑`) | Select track in queue |
| `PageDown` / `PageUp` | Scroll down / up by 10 items (`Ctrl+d` / `Ctrl+u`) |
| `Home` / `End` (`g` / `G`) | Jump to top / bottom of queue |
| `Enter` | Play selected track immediately |
| `d` / `Delete` | Remove selected track from queue |
| `c` | Clear entire playlist queue |

---

## 🚀 Getting Started

### Prerequisites
- Rust 1.80+ (Rust 2024 edition supported)

### Running the Player
From the project folder:
```bash
cargo run --release
```

To run tests:
```bash
cargo test
```
