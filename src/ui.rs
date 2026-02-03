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

// render tui interface
pub fn ui(f: &mut Frame, app: &mut App) {
    let constraints = if app.show_logs {
        vec![
            Constraint::Percentage(70),
            Constraint::Percentage(30),
            Constraint::Length(3),
        ]
    } else {
        vec![Constraint::Fill(1), Constraint::Length(3)]
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

    let items: Vec<ListItem> = if !app.search_query.is_empty() {
        // show filtered notes
        app.notes
            .iter()
            .map(|note| {
                let tags_display = if !note.tags.is_empty() {
                    format!(
                        " [{}]",
                        note.tags
                            .iter()
                            .map(|t| format!("#{}", t))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                } else {
                    String::new()
                };

                let mut spans = Vec::new();

                let separator_idx = note.title.rfind('/').or_else(|| note.title.rfind('\\'));

                if let Some(idx) = separator_idx {
                    let (folder, name) = note.title.split_at(idx + 1);
                    spans.push(Span::styled(
                        format!(" {}", folder),
                        Style::default().fg(app.theme.dim),
                    ));
                    spans.push(Span::raw(name));
                } else {
                    spans.push(Span::raw(format!(" {}", note.title)));
                }

                if !tags_display.is_empty() {
                    spans.push(Span::styled(
                        tags_display,
                        Style::default().fg(app.theme.dim),
                    ));
                }

                ListItem::new(Line::from(spans))
            })
            .collect()
    } else {
        // show file system items
        app.fs_items
            .iter()
            .map(|item| match item {
                crate::data::FileSystemItem::Folder(path) => {
                    let name = path.file_name().unwrap().to_string_lossy();
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            "> ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            name.to_string(),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                    ]))
                }
                crate::data::FileSystemItem::Note(note) => {
                    let name = note.path.file_name().unwrap().to_string_lossy();
                    let name = name.strip_suffix(".md").unwrap_or(&name);

                    ListItem::new(Line::from(vec![
                        Span::styled("  ", Style::default().fg(app.theme.accent)),
                        Span::raw(name.to_string()),
                    ]))
                }
            })
            .collect()
    };

    let title = if !app.search_query.is_empty() {
        format!(" Search Results [{}] ", app.notes.len())
    } else {
        let path_str = if app.current_dir.as_os_str().is_empty() {
            "Root".to_string()
        } else {
            app.current_dir.to_string_lossy().to_string()
        };
        format!(" {} ", path_str)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .title_style(Style::default().add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(app.theme.accent)),
        )
        .highlight_style(
            Style::default()
                .bg(app.theme.selection)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ");

    f.render_stateful_widget(list, main_chunks[0], &mut app.list_state);

    // keep content string alive
    // let mut content_string = String::new();

    let (preview_content, preview_title, preview_footer) = if let Some(i) =
        app.list_state.selected()
    {
        let selected_note = if !app.search_query.is_empty() {
            if i < app.notes.len() {
                Some(app.notes[i].clone())
            } else {
                None
            }
        } else if i < app.fs_items.len() {
            match &app.fs_items[i] {
                crate::data::FileSystemItem::Note(n) => Some(n.clone()),
                crate::data::FileSystemItem::Folder(_) => None,
            }
        } else {
            None
        };

        if let Some(note) = selected_note {
            let content_string = note
                .content
                .clone()
                .unwrap_or_else(|| "Loading...".to_string());
            let content = content_string.as_str();

            let lines: Vec<Line> = content
                .lines()
                .map(|line| {
                    if line.starts_with("# ") {
                        Line::from(Span::styled(
                            line.to_string(),
                            Style::default()
                                .fg(app.theme.header)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else if line.starts_with("## ") {
                        Line::from(Span::styled(
                            line.to_string(),
                            Style::default()
                                .fg(app.theme.accent)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else if line.starts_with("### ") {
                        Line::from(Span::styled(
                            line.to_string(),
                            Style::default()
                                .fg(app.theme.selection)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else if line.starts_with(
                        "`
```",
                    ) {
                        Line::from(Span::styled(
                            line.to_string(),
                            Style::default().fg(app.theme.dim),
                        ))
                    } else if line.starts_with("> ") {
                        Line::from(Span::styled(
                            line.to_string(),
                            Style::default()
                                .fg(Color::Rgb(166, 227, 161))
                                .add_modifier(Modifier::ITALIC),
                        ))
                    } else {
                        Line::from(line.to_string())
                    }
                })
                .collect();

            let title = format!(" {} ", note.title);
            let dt: DateTime<Local> = note.last_modified.into();
            let footer = format!(" {} | {} bytes ", dt.format("%Y-%m-%d %H:%M"), note.size);

            (lines, title, footer)
        } else {
            // handle folder selection or invalid index
            if !app.search_query.is_empty() {
                (vec![Line::from("")], " Preview ".to_string(), String::new())
            } else if i < app.fs_items.len() {
                match &app.fs_items[i] {
                    crate::data::FileSystemItem::Folder(p) => (
                        vec![
                            Line::from(""),
                            Line::from(Span::styled(
                                "  > Folder",
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            )),
                            Line::from(format!("  {}", p.file_name().unwrap().to_string_lossy())),
                            Line::from(""),
                            Line::from("  Press 'l' or Enter to open."),
                        ],
                        " Folder Info ".to_string(),
                        String::new(),
                    ),
                    _ => (vec![Line::from("")], " Preview ".to_string(), String::new()),
                }
            } else {
                (vec![Line::from("")], " Preview ".to_string(), String::new())
            }
        }
    } else {
        (
            vec![Line::from(" Press 'n' to create a new note.")],
            " Kiroku ".to_string(),
            String::new(),
        )
    };

    let preview_block = Block::default()
        .title(preview_title)
        .title_style(Style::default().add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.accent));

    let preview_block = if !preview_footer.is_empty() {
        preview_block
            .title_bottom(Line::from(preview_footer).alignment(ratatui::layout::Alignment::Right))
    } else {
        preview_block
    };

    let preview = Paragraph::new(preview_content)
        .block(preview_block)
        .scroll((app.preview_scroll, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(preview, main_chunks[1]);

    // render logs
    if app.show_logs {
        let tui_sm = TuiLoggerWidget::default()
            .block(
                Block::default()
                    .title(" System Logs ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(app.theme.selection)),
            )
            .output_separator('|')
            .output_timestamp(Some("%H:%M:%S".to_string()))
            .style(Style::default().fg(Color::Reset));
        f.render_widget(tui_sm, chunks[1]);
    }

    // render status bar
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
        InputMode::CreatingFolder => format!("{} CREATING FOLDER: {}", spinner, app.status_msg),
        InputMode::Renaming => format!("{} RENAMING NOTE: {}", spinner, app.status_msg),
        InputMode::ConfirmDelete => format!("{} DELETING NOTE: {}", spinner, app.status_msg),
        InputMode::Search => format!("{} SEARCH: {}", spinner, app.search_query),
        InputMode::TagSearch => format!("{} TAG SEARCH: {}", spinner, app.search_query),
        InputMode::ContentSearch => format!("{} CONTENT SEARCH: {}", spinner, app.search_query),
        InputMode::Help => format!("{} HELP: Press Esc to close", spinner),
    };

    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.dim));

    let status = Paragraph::new(status_text.as_str())
        .block(status_block)
        .style(
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(status, status_area);

    // render popups
    if app.input_mode == InputMode::Editing
        || app.input_mode == InputMode::Renaming
        || app.input_mode == InputMode::CreatingFolder
    {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(Clear, area);

        let title = match app.input_mode {
            InputMode::Editing => " New Note ",
            InputMode::Renaming => " Rename Note ",
            InputMode::CreatingFolder => " New Folder ",
            _ => "",
        };

        let input_block = Block::default()
            .title(title)
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
            .border_style(Style::default().fg(app.theme.bold));

        let text = vec![
            Line::from(vec![
                Span::raw("Are you sure you want to "),
                Span::styled(
                    "DELETE",
                    Style::default()
                        .fg(app.theme.bold)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" this note?"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("(y)es", Style::default().fg(Color::Rgb(166, 227, 161))),
                Span::raw(" / "),
                Span::styled("(n)o", Style::default().fg(app.theme.bold)),
            ]),
        ];

        let confirm_text = Paragraph::new(text)
            .block(confirm_block)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(confirm_text, area);
    }

    if app.input_mode == InputMode::Help {
        let area = centered_rect(60, 60, f.area());
        f.render_widget(Clear, area);

        let help_block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.theme.accent));

        let text = vec![
            Line::from(Span::styled(
                "Navigation",
                Style::default()
                    .fg(app.theme.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  j / k       : Scroll list down / up"),
            Line::from("  h / l       : Go up / Enter folder"),
            Line::from("  Ctrl+j / k  : Scroll preview down / up"),
            Line::from("  Enter       : Edit selected note"),
            Line::from(""),
            Line::from(Span::styled(
                "Actions",
                Style::default()
                    .fg(app.theme.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  n           : New note"),
            Line::from("  f           : New folder"),
            Line::from("  r           : Rename note"),
            Line::from("  d           : Delete note"),
            Line::from("  g           : Sync with git"),
            Line::from("  s           : Cycle sort mode"),
            Line::from("  y           : Copy content to clipboard"),
            Line::from("  Y           : Copy path to clipboard"),
            Line::from(""),
            Line::from(Span::styled(
                "Search",
                Style::default()
                    .fg(app.theme.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  /           : Search notes by title"),
            Line::from("  ?           : Search notes by content"),
            Line::from("  #           : Search notes by tag"),
            Line::from("  Esc         : Clear search / Close popup"),
            Line::from(""),
            Line::from(Span::styled(
                "General",
                Style::default()
                    .fg(app.theme.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  F1          : Toggle this help"),
            Line::from("  t           : Cycle themes"),
            Line::from("  F12         : Toggle logs"),
            Line::from("  q           : Quit"),
        ];

        let help_text = Paragraph::new(text)
            .block(help_block)
            .alignment(ratatui::layout::Alignment::Left)
            .wrap(Wrap { trim: false });

        f.render_widget(help_text, area);
    }
}

// center rect helper
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
