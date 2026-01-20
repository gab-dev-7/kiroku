use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use crate::app::App;

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[0]);

    let items: Vec<ListItem> = app
        .notes
        .iter()
        .map(|note| ListItem::new(note.title.clone()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" kiroku notes ")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Yellow),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, main_chunks[0], &mut app.list_state);

    let current_content = if let Some(i) = app.list_state.selected() {
        if i < app.notes.len() {
             app.notes[i].content.as_str()
        } else {
            ""
        }
    } else {
        "press 'n' to create a new note."
    };

    let preview = Paragraph::new(current_content)
        .block(Block::default().title(" preview ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(preview, main_chunks[1]);

    let status = Paragraph::new(app.status_msg.as_str())
        .block(Block::default().borders(Borders::ALL).title(" status "));
    f.render_widget(status, chunks[1]);
}
