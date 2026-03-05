use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph, Wrap};
use std::collections::BTreeSet;
use std::io::stdout;
use std::time::Duration;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct CommitDraft {
    pub scope: String,
    pub subject: String,
    pub body: String,
}

const COLOR_TEXT: Color = Color::Rgb(251, 224, 195); // #fbe0c3
const COLOR_MUTED: Color = Color::Rgb(125, 142, 149); // #7d8e95
const COLOR_BORDER: Color = Color::Rgb(125, 142, 149); // #7d8e95
const COLOR_ACCENT: Color = Color::Rgb(255, 187, 152); // #ffbb98
const COLOR_SUCCESS: Color = Color::Rgb(251, 224, 195); // #fbe0c3
const COLOR_WARNING: Color = Color::Rgb(255, 187, 152); // #ffbb98

fn char_to_byte_index(text: &str, char_idx: usize) -> usize {
    if char_idx == 0 {
        return 0;
    }
    text.char_indices()
        .nth(char_idx)
        .map_or(text.len(), |(idx, _)| idx)
}

fn display_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

fn display_width_before_cursor(text: &str, cursor: usize) -> usize {
    let byte_idx = char_to_byte_index(text, cursor);
    display_width(&text[..byte_idx])
}

fn move_cursor_left(cursor: &mut usize) {
    *cursor = cursor.saturating_sub(1);
}

fn move_cursor_right(cursor: &mut usize, max_chars: usize) {
    if *cursor < max_chars {
        *cursor += 1;
    }
}

fn insert_char_at(text: &mut String, cursor: &mut usize, c: char) {
    let byte_idx = char_to_byte_index(text, *cursor);
    text.insert(byte_idx, c);
    *cursor += 1;
}

fn delete_char_before_cursor(text: &mut String, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }
    let start = char_to_byte_index(text, *cursor - 1);
    let end = char_to_byte_index(text, *cursor);
    text.replace_range(start..end, "");
    *cursor -= 1;
}

fn delete_char_at_cursor(text: &mut String, cursor: usize) {
    let total = text.chars().count();
    if cursor >= total {
        return;
    }
    let start = char_to_byte_index(text, cursor);
    let end = char_to_byte_index(text, cursor + 1);
    text.replace_range(start..end, "");
}

fn cursor_xy_with_wrap(text: &str, cursor: usize, width: u16) -> (u16, u16) {
    let w = usize::from(width.max(1));
    let mut x: usize = 0;
    let mut y: usize = 0;
    for ch in text.chars().take(cursor) {
        if ch == '\n' {
            x = 0;
            y += 1;
            continue;
        }
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if x + ch_width > w {
            x = 0;
            y += 1;
        }
        x += ch_width;
        if x >= w {
            x = 0;
            y += 1;
        }
    }
    (x as u16, y as u16)
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, ch) in text.chars().enumerate() {
        if ch == '\n' {
            starts.push(i + 1);
        }
    }
    starts
}

fn line_of_cursor(text: &str, cursor: usize) -> usize {
    let mut line = 0usize;
    for ch in text.chars().take(cursor) {
        if ch == '\n' {
            line += 1;
        }
    }
    line
}

fn column_of_cursor(text: &str, cursor: usize) -> usize {
    let chars: Vec<char> = text.chars().take(cursor).collect();
    let mut col = 0usize;
    for ch in chars.into_iter().rev() {
        if ch == '\n' {
            break;
        }
        col += 1;
    }
    col
}

fn line_len(text: &str, line_idx: usize) -> usize {
    let starts = line_starts(text);
    let Some(&start) = starts.get(line_idx) else {
        return 0;
    };
    let end = starts
        .get(line_idx + 1)
        .map(|v| v.saturating_sub(1))
        .unwrap_or_else(|| text.chars().count());
    end.saturating_sub(start)
}

fn set_cursor(frame: &mut ratatui::Frame<'_>, area: Rect, x: u16, y: u16) {
    frame.set_cursor_position((area.x + x, area.y + y));
}

struct TuiSession {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl TuiSession {
    fn start() -> Result<Self> {
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(out);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TuiSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn filter_indices(options: &[String], query: &str) -> Vec<usize> {
    if query.trim().is_empty() {
        return (0..options.len()).collect();
    }

    let q = query.to_lowercase();
    options
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| {
            if item.to_lowercase().contains(&q) {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}

fn make_list_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .title(Line::from(Span::styled(
            title.to_string(),
            Style::default()
                .fg(COLOR_ACCENT)
                .add_modifier(Modifier::BOLD),
        )))
        .border_style(Style::default().fg(COLOR_BORDER))
        .style(Style::default().fg(COLOR_TEXT))
}

fn make_inner_block(title: &str) -> Block<'_> {
    make_list_block(title).padding(Padding::new(1, 1, 1, 0))
}

fn inset_horizontally(area: Rect, inset: u16) -> Rect {
    if area.width <= inset.saturating_mul(2) {
        area
    } else {
        Rect {
            x: area.x + inset,
            y: area.y,
            width: area.width - inset * 2,
            height: area.height,
        }
    }
}

fn make_focus_block(title: &str, focused: bool) -> Block<'_> {
    let border_color = if focused { COLOR_WARNING } else { COLOR_BORDER };
    make_inner_block(title).border_style(Style::default().fg(border_color).add_modifier(
        if focused {
            Modifier::BOLD
        } else {
            Modifier::empty()
        },
    ))
}

fn title_style() -> Style {
    Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD)
}

fn hint_style() -> Style {
    Style::default().fg(COLOR_MUTED)
}

fn text_style() -> Style {
    Style::default().fg(COLOR_TEXT)
}

fn list_highlight_style() -> Style {
    Style::default()
        .fg(COLOR_MUTED)
        .bg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD)
}

fn selected_line(selected: usize, total: usize, query: &str) -> String {
    if query.trim().is_empty() {
        format!("Showing {total} items")
    } else {
        format!("Showing {selected}/{total} matches for \"{query}\"")
    }
}

fn menu_style_line(raw: &str) -> Line<'static> {
    if let Some((cmd, desc)) = raw.split_once(" - ") {
        return Line::from(vec![
            Span::styled(
                format!("{:<7}", cmd.trim()),
                Style::default()
                    .fg(COLOR_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - ", Style::default().fg(COLOR_BORDER)),
            Span::styled(desc.trim().to_string(), hint_style()),
        ]);
    }
    Line::from(Span::styled(raw.to_string(), text_style()))
}

pub fn select_one(prompt: &str, options: &[String]) -> Result<Option<usize>> {
    if options.is_empty() {
        return Ok(None);
    }

    let mut session = TuiSession::start()?;
    let mut query = String::new();
    let mut cursor: usize = 0;

    loop {
        let filtered = filter_indices(options, &query);
        if filtered.is_empty() {
            cursor = 0;
        } else if cursor >= filtered.len() {
            cursor = filtered.len() - 1;
        }

        session.terminal.draw(|frame| {
            let outer = make_list_block("wgit");
            let inner = inset_horizontally(outer.inner(frame.area()), 2);
            frame.render_widget(outer, frame.area());

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(5),
                    Constraint::Length(1),
                ])
                .split(inner);

            let title = Paragraph::new(Line::from(prompt.to_string()))
                .style(title_style());
            frame.render_widget(title, chunks[0]);

            let search = Paragraph::new(format!(
                "Search: {}",
                if query.is_empty() { "(empty)" } else { &query }
            ))
            .style(hint_style());
            frame.render_widget(search, chunks[1]);

            let list_area = chunks[2];
            let hint_area = chunks[3];

            let items: Vec<ListItem> = if filtered.is_empty() {
                vec![ListItem::new(Span::styled("No matches", hint_style()))]
            } else {
                filtered
                    .iter()
                    .map(|idx| ListItem::new(menu_style_line(&options[*idx])))
                    .collect()
            };

            let list = List::new(items)
                .block(make_inner_block("Options"))
                .highlight_style(list_highlight_style());
            let mut state = ListState::default();
            if !filtered.is_empty() {
                state.select(Some(cursor));
            }
            frame.render_stateful_widget(list, list_area, &mut state);

            let hint = Paragraph::new(format!(
                "{}  |  Up/Down move  Enter confirm  Esc cancel",
                selected_line(filtered.len(), options.len(), &query)
            ))
            .style(hint_style());
            frame.render_widget(hint, hint_area);
        })?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => return Ok(None),
                KeyCode::Up => {
                    cursor = cursor.saturating_sub(1);
                }
                KeyCode::Down => {
                    let filtered = filter_indices(options, &query);
                    if !filtered.is_empty() && cursor + 1 < filtered.len() {
                        cursor += 1;
                    }
                }
                KeyCode::Enter => {
                    let filtered = filter_indices(options, &query);
                    if filtered.is_empty() {
                        continue;
                    }
                    return Ok(Some(filtered[cursor]));
                }
                KeyCode::Backspace => {
                    query.pop();
                }
                KeyCode::Char(c)
                    if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
                {
                    query.push(c);
                    cursor = 0;
                }
                _ => {}
            }
        }
    }
}

pub fn select_many(prompt: &str, options: &[String]) -> Result<Vec<usize>> {
    if options.is_empty() {
        return Ok(Vec::new());
    }

    let mut session = TuiSession::start()?;
    let mut selected: BTreeSet<usize> = BTreeSet::new();
    let mut query = String::new();
    let mut cursor: usize = 0;

    loop {
        let filtered = filter_indices(options, &query);
        if filtered.is_empty() {
            cursor = 0;
        } else if cursor >= filtered.len() {
            cursor = filtered.len() - 1;
        }

        session.terminal.draw(|frame| {
            let outer = make_list_block("wgit");
            let inner = inset_horizontally(outer.inner(frame.area()), 2);
            frame.render_widget(outer, frame.area());

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(5),
                    Constraint::Length(1),
                ])
                .split(inner);

            let title = Paragraph::new(Line::from(prompt.to_string())).style(title_style());
            frame.render_widget(title, chunks[0]);

            let search = Paragraph::new(format!(
                "Search: {}",
                if query.is_empty() { "(empty)" } else { &query }
            ))
            .style(hint_style());
            frame.render_widget(search, chunks[1]);

            let list_area = chunks[2];
            let hint_area = chunks[3];

            let items: Vec<ListItem> = if filtered.is_empty() {
                vec![ListItem::new(Span::styled("No matches", hint_style()))]
            } else {
                filtered
                    .iter()
                    .map(|idx| {
                        let checked = selected.contains(idx);
                        let marker = if checked { "✓" } else { "□" };
                        let marker_style = if checked {
                            Style::default()
                                .fg(COLOR_SUCCESS)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(COLOR_BORDER)
                        };
                        let file_style = if checked {
                            Style::default()
                                .fg(COLOR_ACCENT)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(COLOR_ACCENT)
                        };
                        ListItem::new(Line::from(vec![
                            Span::styled(marker.to_string(), marker_style),
                            Span::raw(" "),
                            Span::styled(options[*idx].clone(), file_style),
                        ]))
                    })
                    .collect()
            };

            let list = List::new(items)
                .block(make_inner_block("Options"))
                .highlight_style(list_highlight_style());
            let mut state = ListState::default();
            if !filtered.is_empty() {
                state.select(Some(cursor));
            }
            frame.render_stateful_widget(list, list_area, &mut state);

            let hint = Paragraph::new(format!(
                "{} | Selected: {}  |  Space toggle  Right select all  Left clear all  Enter confirm  Esc cancel",
                selected_line(filtered.len(), options.len(), &query),
                selected.len()
            ))
            .style(hint_style());
            frame.render_widget(hint, hint_area);
        })?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => return Ok(Vec::new()),
                KeyCode::Up => {
                    cursor = cursor.saturating_sub(1);
                }
                KeyCode::Down => {
                    let filtered = filter_indices(options, &query);
                    if !filtered.is_empty() && cursor + 1 < filtered.len() {
                        cursor += 1;
                    }
                }
                KeyCode::Right => {
                    for idx in filter_indices(options, &query) {
                        selected.insert(idx);
                    }
                }
                KeyCode::Left => {
                    for idx in filter_indices(options, &query) {
                        selected.remove(&idx);
                    }
                }
                KeyCode::Char(' ') => {
                    let filtered = filter_indices(options, &query);
                    if filtered.is_empty() {
                        continue;
                    }
                    let idx = filtered[cursor];
                    if selected.contains(&idx) {
                        selected.remove(&idx);
                    } else {
                        selected.insert(idx);
                    }
                }
                KeyCode::Enter => return Ok(selected.into_iter().collect()),
                KeyCode::Backspace => {
                    query.pop();
                    cursor = 0;
                }
                KeyCode::Char(c)
                    if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
                {
                    query.push(c);
                    cursor = 0;
                }
                _ => {}
            }
        }
    }
}

pub fn input_text(prompt: &str) -> Result<String> {
    let mut session = TuiSession::start()?;
    let mut value = String::new();
    let mut cursor = 0usize;

    loop {
        session.terminal.draw(|frame| {
            let outer = make_list_block("wgit");
            let inner = inset_horizontally(outer.inner(frame.area()), 2);
            frame.render_widget(outer, frame.area());

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(4),
                    Constraint::Length(1),
                ])
                .split(inner);

            let title = Paragraph::new(prompt).style(title_style());
            frame.render_widget(title, chunks[0]);

            let input_block = make_inner_block("Input");
            let input_inner = input_block.inner(chunks[1]);
            let input_width = input_inner.width.max(1);
            let total_cursor_x =
                (display_width("> ") + display_width_before_cursor(&value, cursor)) as u16;
            let scroll_x = total_cursor_x.saturating_sub(input_width.saturating_sub(1));
            let input = Paragraph::new(format!("> {value}"))
                .scroll((0, scroll_x))
                .style(text_style())
                .block(input_block);
            frame.render_widget(input, chunks[1]);
            let display_x = total_cursor_x.saturating_sub(scroll_x);
            set_cursor(frame, input_inner, display_x, 0);

            let hint = Paragraph::new(
                "Enter confirm  Esc cancel  Left/Right move  Home/End line start/end",
            )
            .style(hint_style());
            frame.render_widget(hint, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => return Ok(value),
                KeyCode::Esc => return Ok(String::new()),
                KeyCode::Backspace => {
                    delete_char_before_cursor(&mut value, &mut cursor);
                }
                KeyCode::Delete => {
                    delete_char_at_cursor(&mut value, cursor);
                }
                KeyCode::Left => {
                    move_cursor_left(&mut cursor);
                }
                KeyCode::Right => {
                    move_cursor_right(&mut cursor, value.chars().count());
                }
                KeyCode::Home => {
                    cursor = 0;
                }
                KeyCode::End => {
                    cursor = value.chars().count();
                }
                KeyCode::Char(c)
                    if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
                {
                    insert_char_at(&mut value, &mut cursor, c);
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CommitField {
    Scope,
    Subject,
    Body,
}

impl CommitField {
    fn up(self) -> Self {
        match self {
            Self::Scope => Self::Scope,
            Self::Subject => Self::Scope,
            Self::Body => Self::Subject,
        }
    }

    fn down(self) -> Self {
        match self {
            Self::Scope => Self::Subject,
            Self::Subject => Self::Body,
            Self::Body => Self::Body,
        }
    }
}

pub fn edit_commit_message(commit_type: &str) -> Result<Option<CommitDraft>> {
    let mut session = TuiSession::start()?;
    let mut scope = String::new();
    let mut subject = String::new();
    let mut body = String::new();
    let mut scope_cursor = 0usize;
    let mut subject_cursor = 0usize;
    let mut body_cursor = 0usize;
    let mut active = CommitField::Subject;

    loop {
        session.terminal.draw(|frame| {
            let outer = make_list_block("wgit");
            let inner = inset_horizontally(outer.inner(frame.area()), 2);
            frame.render_widget(outer, frame.area());

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Min(7),
                    Constraint::Length(1),
                ])
                .split(inner);

            let header = Paragraph::new(format!(
                "Commit editor ({commit_type})  |  Esc save  Ctrl+C cancel"
            ))
            .style(title_style());
            frame.render_widget(header, chunks[0]);

            let scope_block = make_focus_block("Scope", active == CommitField::Scope);
            let scope_inner = scope_block.inner(chunks[1]);
            let scope_label = "scope: ";
            let scope_line = Line::from(vec![
                Span::styled(
                    scope_label.to_string(),
                    Style::default().fg(COLOR_WARNING).add_modifier(Modifier::BOLD),
                ),
                Span::styled(scope.clone(), text_style()),
            ]);
            let scope_total_x =
                (display_width(scope_label) + display_width_before_cursor(&scope, scope_cursor)) as u16;
            let scope_scroll_x = scope_total_x.saturating_sub(scope_inner.width.saturating_sub(1));
            let scope_paragraph = Paragraph::new(scope_line)
                .scroll((0, scope_scroll_x))
                .block(scope_block);
            frame.render_widget(scope_paragraph, chunks[1]);

            let subject_block = make_focus_block("Subject", active == CommitField::Subject);
            let subject_inner = subject_block.inner(chunks[2]);
            let subject_label = "subject: ";
            let subject_line = Line::from(vec![
                Span::styled(
                    subject_label.to_string(),
                    Style::default().fg(COLOR_WARNING).add_modifier(Modifier::BOLD),
                ),
                Span::styled(subject.clone(), text_style()),
            ]);
            let subject_total_x = (display_width(subject_label)
                + display_width_before_cursor(&subject, subject_cursor)) as u16;
            let subject_scroll_x =
                subject_total_x.saturating_sub(subject_inner.width.saturating_sub(1));
            let subject_paragraph = Paragraph::new(subject_line)
                .scroll((0, subject_scroll_x))
                .block(subject_block);
            frame.render_widget(subject_paragraph, chunks[2]);

            let body_block = make_focus_block("Body", active == CommitField::Body);
            let body_inner = body_block.inner(chunks[3]);
            let (body_cursor_x, body_cursor_y) =
                cursor_xy_with_wrap(&body, body_cursor, body_inner.width.max(1));
            let body_scroll_y = body_cursor_y.saturating_sub(body_inner.height.saturating_sub(1));
            let body_paragraph = Paragraph::new(body.clone())
                .scroll((body_scroll_y, 0))
                .wrap(Wrap { trim: false })
                .style(text_style())
                .block(body_block);
            frame.render_widget(body_paragraph, chunks[3]);

            let hint = Paragraph::new(
                "Up/Down switch field  Enter next/newline  Left/Right move  Home/End line start/end",
            )
            .style(hint_style());
            frame.render_widget(hint, chunks[4]);

            match active {
                CommitField::Scope => {
                    let display_x = scope_total_x.saturating_sub(scope_scroll_x);
                    set_cursor(frame, scope_inner, display_x, 0);
                }
                CommitField::Subject => {
                    let display_x = subject_total_x.saturating_sub(subject_scroll_x);
                    set_cursor(frame, subject_inner, display_x, 0);
                }
                CommitField::Body => {
                    set_cursor(frame, body_inner, body_cursor_x, body_cursor_y.saturating_sub(body_scroll_y));
                }
            }
        })?;

        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match (code, modifiers) {
                (KeyCode::Esc, _) => {
                    return Ok(Some(CommitDraft {
                        scope,
                        subject,
                        body,
                    }));
                }
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(None),
                (KeyCode::Up, _) => {
                    active = active.up();
                }
                (KeyCode::Down, _) => {
                    active = active.down();
                }
                (KeyCode::Enter, _) => match active {
                    CommitField::Scope => active = CommitField::Subject,
                    CommitField::Subject => active = CommitField::Body,
                    CommitField::Body => insert_char_at(&mut body, &mut body_cursor, '\n'),
                },
                (KeyCode::Backspace, _) => match active {
                    CommitField::Scope => delete_char_before_cursor(&mut scope, &mut scope_cursor),
                    CommitField::Subject => {
                        delete_char_before_cursor(&mut subject, &mut subject_cursor)
                    }
                    CommitField::Body => delete_char_before_cursor(&mut body, &mut body_cursor),
                },
                (KeyCode::Delete, _) => match active {
                    CommitField::Scope => delete_char_at_cursor(&mut scope, scope_cursor),
                    CommitField::Subject => delete_char_at_cursor(&mut subject, subject_cursor),
                    CommitField::Body => delete_char_at_cursor(&mut body, body_cursor),
                },
                (KeyCode::Left, _) => match active {
                    CommitField::Scope => move_cursor_left(&mut scope_cursor),
                    CommitField::Subject => move_cursor_left(&mut subject_cursor),
                    CommitField::Body => move_cursor_left(&mut body_cursor),
                },
                (KeyCode::Right, _) => match active {
                    CommitField::Scope => {
                        move_cursor_right(&mut scope_cursor, scope.chars().count())
                    }
                    CommitField::Subject => {
                        move_cursor_right(&mut subject_cursor, subject.chars().count())
                    }
                    CommitField::Body => move_cursor_right(&mut body_cursor, body.chars().count()),
                },
                (KeyCode::Home, _) => match active {
                    CommitField::Scope => scope_cursor = 0,
                    CommitField::Subject => subject_cursor = 0,
                    CommitField::Body => {
                        let col = column_of_cursor(&body, body_cursor);
                        body_cursor = body_cursor.saturating_sub(col);
                    }
                },
                (KeyCode::End, _) => match active {
                    CommitField::Scope => scope_cursor = scope.chars().count(),
                    CommitField::Subject => subject_cursor = subject.chars().count(),
                    CommitField::Body => {
                        let col = column_of_cursor(&body, body_cursor);
                        let line = line_of_cursor(&body, body_cursor);
                        let len = line_len(&body, line);
                        body_cursor = body_cursor.saturating_sub(col) + len;
                    }
                },
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => match active {
                    CommitField::Scope => insert_char_at(&mut scope, &mut scope_cursor, c),
                    CommitField::Subject => insert_char_at(&mut subject, &mut subject_cursor, c),
                    CommitField::Body => insert_char_at(&mut body, &mut body_cursor, c),
                },
                _ => {}
            }
        }
    }
}

pub fn confirm(prompt: &str) -> Result<bool> {
    let options = vec!["yes".to_string(), "no".to_string()];
    let choice = select_one(prompt, &options)?;
    Ok(matches!(choice, Some(0)))
}
