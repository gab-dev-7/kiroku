use crate::app::{App, InputMode};
use chrono::{DateTime, Local};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use tui_logger::TuiLoggerWidget;

const ACCENT_COLOR: Color = Color::Rgb(137, 220, 235);
const SELECTION_COLOR: Color = Color::Rgb(187, 154, 247)
const DIM_COLOR: Color = Color::Rgb(108, 112, 134);
const HEADER_COLOR: Color = Color::Rgb(137, 180, 250);
const BOLD_COLOR: Color = Color::Rgb(243, 139, 168);

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

    let main_area = chunks[0];
    let status_area = if app.show_logs { chunks[2] } else { chunks[1] };

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(main_area);

    // --- List Widget ---
    let items: Vec<ListItem> = app
        .notes
        .iter()
        .map(|note| ListItem::new(format!(" {} ", note.title)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Notes ")
                .title_style(Style::default().add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(ACCENT_COLOR)),
        )
        .highlight_style(
            Style::default()
                .bg(SELECTION_COLOR)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ");

    f.render_stateful_widget(list, main_chunks[0], &mut app.list_state);

    // Preview Widget
    let (preview_content, preview_title, preview_footer) =
        if let Some(i) = app.list_state.selected() {
            if i < app.notes.len() {
                let note = &app.notes[i];
                let content = note.content.as_deref().unwrap_or("Loading...");

                let lines: Vec<Line> = content
                    .lines()
                    .map(|line| {
                        if line.starts_with("# ") {
                            Line::from(Span::styled(
                                line,
                                Style::default()
                                    .fg(HEADER_COLOR)
                                    .add_modifier(Modifier::BOLD),
                            ))
                        } else if line.starts_with("## ") {
                            Line::from(Span::styled(
                                line,
                                Style::default()
                                    .fg(ACCENT_COLOR)
                                    .add_modifier(Modifier::BOLD),
                            ))
                        } else if line.starts_with("### ") {
                            Line::from(Span::styled(
                                line,
                                Style::default()
                                    .fg(SELECTION_COLOR)
                                    .add_modifier(Modifier::BOLD),
                            ))
                        } else if line.starts_with(
                            "`
```",
                        ) {
                            Line::from(Span::styled(line, Style::default().fg(DIM_COLOR)))
                        } else if line.starts_with("> ") {
                            Line::from(Span::styled(
                                line,
                                Style::default()
                                    .fg(Color::Rgb(166, 227, 161))
                                    .add_modifier(Modifier::ITALIC),
                            ))
                        } else {
                            Line::from(line)
                        }
                    })
                    .collect();

                let title = format!(" {} ", note.title);
                let dt: DateTime<Local> = note.last_modified.into();
                let footer = format!(" {} | {} bytes ", dt.format("%Y-%m-%d %H:%M"), note.size);

                (lines, title, footer)
            } else {
                (vec![Line::from("")], " Preview ".to_string(), String::new())
            }
        } else {
            (
                vec![Line::from(" Press 'n' to create a new note.")],
                " Kiroku ".to_string(),
                String::new(),
            )
        };

    let preview_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(main_chunks[1]);

    let preview = Paragraph::new(preview_content)
        .block(
            Block::default()
                .title(preview_title)
                .title_style(Style::default().add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(ACCENT_COLOR)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(preview, preview_chunks[0]);

    if !preview_footer.is_empty() {
        let footer_block = Block::default()
            .borders(Borders::TOP)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(DIM_COLOR));

        let footer = Paragraph::new(preview_footer)
            .block(footer_block)
            .style(Style::default().fg(DIM_COLOR))
            .alignment(ratatui::layout::Alignment::Right);
        f.render_widget(footer, preview_chunks[1]);
    }

    // Logs
    if app.show_logs {
        let tui_sm = TuiLoggerWidget::default()
            .block(
                Block::default()
                    .title(" System Logs ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(SELECTION_COLOR)),
            )
            .output_separator('|')
            .output_timestamp(Some("%H:%M:%S".to_string()))
            .style(Style::default().fg(Color::Reset));
        f.render_widget(tui_sm, chunks[1]);
    }

    // Status Bar
    let spinner = if app.syncing {
        let frames = ["|", "/", "-", "\\"];
        format!(" {} ", frames[app.spinner_index])
    } else {
        String::new()
    };

    let status_text = match app.input_mode {
        InputMode::Normal => {
            if !app.search_query.is_empty() {
                format!(
                    "{} Filtered: '{}' (Esc to clear)",
                    spinner, app.search_query
                )
            } else {
                format!("{}{}", spinner, app.status_msg)
            }
        }
        InputMode::Editing => format!("{} CREATING NOTE: {}", spinner, app.status_msg),
        InputMode::ConfirmDelete => format!("{} DELETING NOTE: {}", spinner, app.status_msg),
        InputMode::Search => format!("{} SEARCH: {}", spinner, app.search_query),
    };

    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM_COLOR));

    let status = Paragraph::new(status_text.as_str())
        .block(status_block)
        .style(
            Style::default()
                .fg(ACCENT_COLOR)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(status, status_area);

    // Popups
    if app.input_mode == InputMode::Editing {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(Clear, area);

        let input_block = Block::default()
            .title(" New Note ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(166, 227, 161)));

        let input_text = Paragraph::new(app.input.as_str())
            .block(input_block)
            .style(
                Style::default()
                    .fg(Color::Reset)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(input_text, area);
    }

    if app.input_mode == InputMode::ConfirmDelete {
        let area = centered_rect(40, 20, f.area());
        f.render_widget(Clear, area);

        let confirm_block = Block::default()
            .title(" Delete Note ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BOLD_COLOR));

        let text = vec![
            Line::from(vec![
                Span::raw("Are you sure you want to "),
                Span::styled(
                    "DELETE",
                    Style::default().fg(BOLD_COLOR).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" this note?"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("(y)es", Style::default().fg(Color::Rgb(166, 227, 161))),
                Span::raw(" / "),
                Span::styled("(n)o", Style::default().fg(BOLD_COLOR)),
            ]),
        ];

        let confirm_text = Paragraph::new(text)
            .block(confirm_block)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(confirm_text, area);
    }
}

// Helper to center popups
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

