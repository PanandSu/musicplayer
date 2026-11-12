use crate::app::{ActivePanel, App};
use crate::file_browser::ItemType;
use crate::metadata::format_duration;
use crate::playlist::RepeatMode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};
use std::time::Duration;

pub fn render_ui(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Main vertical layout: Header, Body, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // File browser & Playlist/Visualizer
            Constraint::Length(4), // Progress & Status bar
        ])
        .split(size);

    render_header(frame, app, chunks[0]);
    render_body(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);

    if app.show_help {
        render_help_modal(frame, size);
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let play_state = if app.audio.is_playing() {
        Span::styled(
            " ▶ PLAYING ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else if app.audio.is_paused() {
        Span::styled(
            " ⏸ PAUSED ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " ⏹ STOPPED ",
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
    };

    let repeat_state = match app.playlist.repeat_mode() {
        RepeatMode::Off => Span::styled(" [🔁 Repeat: Off] ", Style::default().fg(Color::DarkGray)),
        RepeatMode::All => Span::styled(" [🔁 Repeat: All] ", Style::default().fg(Color::Cyan)),
        RepeatMode::One => Span::styled(
            " [🔂 Repeat: Track] ",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        ),
    };

    let shuffle_state = if app.playlist.is_shuffle() {
        Span::styled(
            " [🔀 Shuffle: ON] ",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" [🔀 Shuffle: OFF] ", Style::default().fg(Color::DarkGray))
    };

    let volume_state = if app.audio.is_muted() {
        Span::styled(
            " [🔇 MUTED] ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            format!(" [🔊 {:.0}%] ", app.audio.volume() * 100.0),
            Style::default().fg(Color::Cyan),
        )
    };

    let header_line = Line::from(vec![
        Span::styled(
            " 🎵 Ratatui Music Player ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        play_state,
        Span::raw(" "),
        repeat_state,
        Span::raw(" "),
        shuffle_state,
        Span::raw(" "),
        volume_state,
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let header_para = Paragraph::new(header_line)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(header_para, area);
}

fn render_body(frame: &mut Frame, app: &mut App, area: Rect) {
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(42), // Left: File Browser
            Constraint::Percentage(58), // Right: Playlist & Visualizer
        ])
        .split(area);

    render_file_browser(frame, app, h_chunks[0]);
    render_right_panel(frame, app, h_chunks[1]);
}

fn render_file_browser(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.active_panel == ActivePanel::FileBrowser;
    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let current_dir_name = app
        .browser
        .current_dir
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("/");

    let filtered = app.browser.filtered_items();
    let filtered_len = filtered.len();
    let current_pos = if filtered_len == 0 {
        0
    } else {
        (app.browser.selected_idx + 1).min(filtered_len)
    };

    let title = format!(
        " 📁 Files: {} ({}/{}) {} ",
        current_dir_name,
        current_pos,
        filtered_len,
        if app.is_searching {
            format!("(Filter: '{}')", app.search_query)
        } else {
            String::new()
        }
    );

    let mut items = Vec::new();

    for (idx, item) in filtered.iter().enumerate() {
        let is_selected = idx == app.browser.selected_idx;

        let (icon, color) = match &item.item_type {
            ItemType::ParentDir => (" ⤴  ..", Color::Blue),
            ItemType::Directory => (" 📁 ", Color::Blue),
            ItemType::AudioFile(ext) => match ext.as_str() {
                "FLAC" => (" 💽 [FLAC] ", Color::Green),
                "MP3" => (" 🎵 [MP3]  ", Color::Cyan),
                "WAV" => (" 🎼 [WAV]  ", Color::Magenta),
                "OGG" => (" 🎷 [OGG]  ", Color::Yellow),
                _ => (" 🎧 [AUD]  ", Color::White),
            },
        };

        let line = Line::from(vec![
            Span::styled(
                if is_selected { "▶ " } else { "  " },
                Style::default().fg(if is_selected {
                    Color::Yellow
                } else {
                    Color::DarkGray
                }),
            ),
            Span::styled(icon, Style::default().fg(color)),
            Span::styled(
                item.name.clone(),
                if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            ),
        ]);

        let list_item = if is_selected && is_focused {
            ListItem::new(line).style(
                Style::default()
                    .bg(Color::Rgb(30, 45, 60))
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            ListItem::new(line)
        };

        items.push(list_item);
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            title,
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(border_color));

    app.browser_state.select(if filtered_len == 0 {
        None
    } else {
        Some(app.browser.selected_idx)
    });

    let list = List::new(items).block(block);
    frame.render_stateful_widget(list, area, &mut app.browser_state);

    if filtered_len > 0 {
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(filtered_len)
            .position(app.browser_state.offset());
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

fn render_right_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(52), // Playlist Queue
            Constraint::Percentage(48), // Now Playing + Visualizer
        ])
        .split(area);

    render_playlist(frame, app, v_chunks[0]);
    render_now_playing_and_visualizer(frame, app, v_chunks[1]);
}

fn render_playlist(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.active_panel == ActivePanel::Playlist;
    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let total_tracks = app.playlist.len();
    let current_idx = app.playlist.current_index();
    let selected_idx = app.playlist.selected_index();

    let current_pos = if total_tracks == 0 {
        0
    } else {
        (selected_idx + 1).min(total_tracks)
    };
    let title = format!(" 📜 Queue: ({}/{}) ", current_pos, total_tracks);

    let items: Vec<ListItem> = app
        .playlist
        .tracks()
        .iter()
        .enumerate()
        .map(|(idx, track)| {
            let is_current = Some(idx) == current_idx;
            let is_selected = idx == selected_idx;

            let marker = if is_current && app.audio.is_playing() {
                " ▶ "
            } else if is_current {
                " ⏸ "
            } else if is_selected {
                " > "
            } else {
                "   "
            };

            let title_str = track.metadata.display_title();
            let artist_str = track.metadata.display_artist();
            let duration_str = format_duration(track.metadata.duration);

            let style = if is_current {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            let line = Line::from(vec![
                Span::styled(
                    marker,
                    if is_current {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Yellow)
                    },
                ),
                Span::styled(format!("{:2}. ", idx + 1), Style::default().fg(Color::DarkGray)),
                Span::styled(title_str, style),
                Span::styled(format!(" - {}", artist_str), Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(" [{}]", duration_str),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            if is_selected && is_focused {
                ListItem::new(line).style(
                    Style::default()
                        .bg(Color::Rgb(35, 45, 65))
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            title,
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(border_color));

    app.playlist_state.select(if total_tracks == 0 {
        None
    } else {
        Some(selected_idx)
    });

    let list = List::new(items).block(block);
    frame.render_stateful_widget(list, area, &mut app.playlist_state);

    if total_tracks > 0 {
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(total_tracks)
            .position(app.playlist_state.offset());
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

fn render_now_playing_and_visualizer(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.active_panel == ActivePanel::Visualizer;
    let border_color = if is_focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    let title = if is_focused {
        format!(
            " 🎧 Visualizer: {} [Press <Tab> to switch] ",
            app.visualizer.style.display_name()
        )
    } else {
        format!(
            " 🎧 Visualizer: {} ",
            app.visualizer.style.display_name()
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            title,
            Style::default()
                .fg(if is_focused { Color::Yellow } else { Color::Magenta })
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 4 {
        return;
    }

    let sub_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Info details
            Constraint::Min(3),    // Spectrum visualizer
        ])
        .split(inner);

    // Track metadata info
    let info_lines = if let Some(track) = app.playlist.current_track() {
        let meta = &track.metadata;
        let sample_rate_str = meta
            .sample_rate
            .map(|s| format!("{:.1} kHz", s as f64 / 1000.0))
            .unwrap_or_else(|| "N/A".to_string());
        let bitrate_str = meta
            .bitrate
            .map(|b| format!("{} kbps", b))
            .unwrap_or_else(|| "N/A".to_string());

        vec![
            Line::from(vec![
                Span::styled("Title: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    meta.display_title(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled("Artist: ", Style::default().fg(Color::DarkGray)),
                Span::styled(meta.display_artist(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Album: ", Style::default().fg(Color::DarkGray)),
                Span::styled(meta.display_album(), Style::default().fg(Color::Gray)),
                Span::raw(" | "),
                Span::styled(
                    format!("[{}] {} @ {}", meta.format, sample_rate_str, bitrate_str),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ]
    } else {
        vec![
            Line::from(Span::styled(
                "No track currently playing. Select a song and press Enter/Space.",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    };

    let info_para = Paragraph::new(info_lines);
    frame.render_widget(info_para, sub_chunks[0]);

    // Spectrum visualizer
    render_spectrum(frame, app, sub_chunks[1]);
}

fn render_spectrum(frame: &mut Frame, app: &mut App, area: Rect) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let lines = app.visualizer.render(area.width as usize, area.height as usize);
    let spectrum_para = Paragraph::new(lines);
    frame.render_widget(spectrum_para, area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Progress bar
            Constraint::Length(2), // Status & Shortcuts
        ])
        .split(area);

    // Progress Bar
    let (curr_dur, total_dur, ratio) = if let Some(track) = app.playlist.current_track() {
        let pos = app.audio.get_position();
        let total = track.metadata.duration;
        let r = if total.as_secs_f64() > 0.0 {
            (pos.as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (pos, total, r)
    } else {
        (Duration::ZERO, Duration::ZERO, 0.0)
    };

    let progress_label = format!(
        "{} / {}  ({:.0}%)",
        format_duration(curr_dur),
        format_duration(total_dur),
        ratio * 100.0
    );

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default()),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::Rgb(30, 30, 45))
                .add_modifier(Modifier::BOLD),
        )
        .ratio(ratio)
        .label(Span::styled(
            progress_label,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));

    frame.render_widget(gauge, chunks[0]);

    // Status bar & Hotkeys hint
    let status_text = app.current_status().map(|s| s.to_string()).unwrap_or_else(|| {
        if app.active_panel == ActivePanel::Visualizer {
            "[Tab] Next Visualizer Style  [v] Switch Style  [h/Esc] Files  [k] Queue  [Space] Play/Pause".to_string()
        } else {
            "[Space] Play/Pause [Tab] Switch Panel [v] Visualizer Style [n/p] Next/Prev [←/→] Seek [+/-] Vol [?] Help".to_string()
        }
    });

    let status_para = Paragraph::new(Line::from(vec![
        Span::styled(" ✨ ", Style::default().fg(Color::Yellow)),
        Span::styled(
            status_text,
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        ),
    ]));

    frame.render_widget(status_para, chunks[1]);
}

fn render_help_modal(frame: &mut Frame, area: Rect) {
    let width = 68.min(area.width.saturating_sub(4));
    let height = 26.min(area.height.saturating_sub(2));

    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let modal_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(Span::styled(
            " ❓ Keyboard Controls & Shortcuts ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = vec![
        Line::from(vec![
            Span::styled(" [General Controls] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Space        Play / Pause audio playback"),
        Line::from("  Tab          Switch focus between File Browser & Playlist"),
        Line::from("  n / p        Skip to Next / Previous track"),
        Line::from("  Left / Right Seek backward / forward 5 seconds"),
        Line::from("  + / -        Increase / Decrease playback volume"),
        Line::from("  m            Toggle Mute / Unmute"),
        Line::from("  s            Toggle Shuffle mode (ON / OFF)"),
        Line::from("  r            Cycle Repeat mode (Off -> All -> Track)"),
        Line::from("  v / Tab (in Vis) Switch Visualizer Style (Bars, Sparks, Braille, Retro EQ, Warp)"),
        Line::from("  Ctrl + L     Redraw & refresh terminal screen"),
        Line::from("  ?            Toggle this Help window"),
        Line::from("  q / Esc      Quit application / Close Help"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [Navigation & Scrolling] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  j / k (↓/↑)  Navigate down / up"),
        Line::from("  PgDn / PgUp  Scroll down / up by 10 items (Ctrl+d / Ctrl+u)"),
        Line::from("  Home / End   Jump to top / bottom (g / G)"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [File Browser Navigation] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  h (Backsp.)  Go up to parent directory"),
        Line::from("  Enter / l    Open folder OR add & play selected audio"),
        Line::from("  a            Add selected audio track to playlist queue"),
        Line::from("  A            Add ALL audio tracks in current folder to playlist"),
        Line::from("  /            Search / filter files by name"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [Playlist Queue Management] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Enter        Play selected track immediately"),
        Line::from("  d (Delete)   Remove selected track from queue"),
        Line::from("  c            Clear all tracks from queue"),
    ];

    let para = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(para, modal_area);
}
