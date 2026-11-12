use musicplayer::app::App;
use musicplayer::file_browser::{BrowserItem, ItemType};
use musicplayer::ui::render_ui;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

#[test]
fn test_browser_scrolling_past_39_items() {
    let mut app = App::new().expect("Failed to create app");

    // Populate browser with 60 dummy items
    app.browser.items.clear();
    for i in 0..60 {
        app.browser.items.push(BrowserItem {
            name: format!("Track_{:02}.flac", i),
            path: PathBuf::from(format!("/music/Track_{:02}.flac", i)),
            item_type: ItemType::AudioFile("FLAC".to_string()),
        });
    }

    // Navigate to item 45
    for _ in 0..45 {
        app.browser.select_next();
    }
    assert_eq!(app.browser.selected_idx, 45);

    // Render in a 30-row terminal window (smaller than 45 items)
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            render_ui(f, &mut app);
        })
        .unwrap();

    // Verify browser_state scrolled offset so that item 45 is in view
    let offset = app.browser_state.offset();
    println!("File Browser ListState offset after selecting item 45: {}", offset);
    assert!(
        offset > 20,
        "Expected list to scroll down past offset 20, got: {}",
        offset
    );

    // Verify terminal buffer actually displays Track_45
    let buffer = terminal.backend().buffer().clone();
    let rendered_text = format!("{:?}", buffer);
    assert!(
        rendered_text.contains("Track_45.flac"),
        "Rendered buffer should contain Track_45.flac"
    );
}

#[test]
fn test_playlist_scrolling_past_39_items() {
    let mut app = App::new().expect("Failed to create app");

    // Add 60 dummy files to playlist
    let paths: Vec<PathBuf> = (0..60)
        .map(|i| PathBuf::from(format!("Song_{:02}.flac", i)))
        .collect();
    app.playlist.add_files(&paths);

    // Jump to item 50 using select_page_down
    for _ in 0..5 {
        app.playlist.select_page_down(10);
    }
    assert_eq!(app.playlist.selected_index(), 50);

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            render_ui(f, &mut app);
        })
        .unwrap();

    let offset = app.playlist_state.offset();
    println!("Playlist ListState offset after selecting track 50: {}", offset);
    assert!(
        offset > 30,
        "Expected playlist to scroll down past offset 30, got: {}",
        offset
    );

    let buffer = terminal.backend().buffer().clone();
    let rendered_text = format!("{:?}", buffer);
    assert!(
        rendered_text.contains("Song_49"),
        "Rendered buffer should contain Song_49"
    );
}
