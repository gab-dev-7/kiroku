use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use tui_logger::TuiLoggerWidget;

pub fn ui(f: &mut Frame, app: &mut App) {
    let constraints = if app.show_logs {
        vec![
            Constraint::Percentage(70),
            Constraint::Percentage(30),
            Constraint::Length(3),
        ]
    } else {
        vec![Constraint::Min(0), Constraint::Length(3)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.area());

    // Main content (List + Preview)
    let main_area = chunks[0];
    let status_area = if app.show_logs { chunks[2] } else { chunks[1] };

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(main_area);

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
            app.notes[i].content.as_deref().unwrap_or("Loading...")
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

    // Logs (if enabled)
    if app.show_logs {
        let tui_sm = TuiLoggerWidget::default()
            .block(
                Block::default()
                    .title(" Logs ")
                    .border_style(Style::default().fg(Color::White).bg(Color::Black))
                    .borders(Borders::ALL),
            )
            .output_separator('|')
            .output_timestamp(Some("%H:%M:%S".to_string()))
            .output_level(Some(tui_logger::TuiLoggerLevelOutput::Long))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .style(Style::default().fg(Color::White).bg(Color::Black));
        f.render_widget(tui_sm, chunks[1]);
    }

    // Status Bar
    let status = Paragraph::new(app.status_msg.as_str())
        .block(Block::default().borders(Borders::ALL).title(" status "));
    f.render_widget(status, status_area);
}
