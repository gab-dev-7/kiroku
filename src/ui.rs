use crate::app::{App, InputMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
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
    let status_text = match app.input_mode {
        InputMode::Normal => {
            if !app.search_query.is_empty() {
                format!("Filtered by: '{}' (Esc to clear)", app.search_query)
            } else {
                app.status_msg.clone()
            }
        }
        InputMode::Editing => format!("CREATING NOTE: {}", app.status_msg),
        InputMode::ConfirmDelete => format!("DELETING NOTE: {}", app.status_msg),
        InputMode::Search => format!("SEARCH: {}", app.search_query),
    };

    let status = Paragraph::new(status_text.as_str())
        .block(Block::default().borders(Borders::ALL).title(" status "));
    f.render_widget(status, status_area);

    // Popups
    if app.input_mode == InputMode::Editing {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(Clear, area); // Clear background

        let input_block = Block::default()
            .title(" New Note Filename ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Blue).fg(Color::White));

        let input_text = Paragraph::new(app.input.as_str())
            .block(input_block)
            .style(Style::default().fg(Color::White));

        f.render_widget(input_text, area);
    }

    if app.input_mode == InputMode::ConfirmDelete {
        let area = centered_rect(40, 20, f.area());
        f.render_widget(Clear, area);

        let confirm_block = Block::default()
            .title(" Confirm Delete ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Red).fg(Color::White));

        let text = format!("Are you sure you want to delete this note?\n\n(y)es / (n)o");
        let confirm_text = Paragraph::new(text)
            .block(confirm_block)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(confirm_text, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1]);

    layout[1]
}
