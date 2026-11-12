use ratatui::{
    backend::TestBackend,
    layout::Rect,
    widgets::{List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Terminal,
};

#[test]
fn test_stateful_list_and_scrollbar() {
    let backend = TestBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let items: Vec<ListItem> = (0..50)
        .map(|i| ListItem::new(format!("Item {}", i)))
        .collect();

    let mut state = ListState::default();
    state.select(Some(45));

    terminal
        .draw(|f| {
            let list = List::new(items.clone());
            f.render_stateful_widget(list, Rect::new(0, 0, 40, 10), &mut state);

            let mut scrollbar_state = ScrollbarState::default()
                .content_length(50)
                .position(state.offset());
            let scrollbar = Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);
            f.render_stateful_widget(scrollbar, Rect::new(0, 0, 40, 10), &mut scrollbar_state);
        })
        .unwrap();

    assert!(state.offset() > 30);
}
