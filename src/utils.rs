use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    queue,
    style::{Color as CrossColor, Print, ResetColor, SetForegroundColor, SetAttribute, Attribute},
    terminal::{self, ClearType},
};
use std::io::{stdout, Write};

pub fn get_theme() -> inquire::ui::RenderConfig<'static> {
    use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};

    let selected_style = StyleSheet::new().with_fg(Color::LightBlue);
    let help_style = StyleSheet::new().with_fg(Color::DarkGrey);
    let checkbox_style = StyleSheet::new().with_fg(Color::LightGreen);
    
    RenderConfig::default()
        .with_selected_option(Some(selected_style))
        .with_selected_checkbox(Styled::new("✔").with_style_sheet(checkbox_style))
        .with_unselected_checkbox(Styled::new("☐").with_fg(Color::DarkGrey))
        .with_scroll_up_prefix(Styled::new("▲").with_fg(Color::LightBlue))
        .with_scroll_down_prefix(Styled::new("▼").with_fg(Color::LightBlue))
        .with_prompt_prefix(Styled::new("?").with_fg(Color::LightGreen))
        .with_help_message(help_style)
}

struct RawModeGuard;

impl RawModeGuard {
    fn init() -> anyhow::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

pub fn run_commit_form() -> anyhow::Result<(String, String, String)> {
    let _guard = RawModeGuard::init()?;
    let mut stdout = stdout();
    
    let mut scope = String::new();
    let mut subject = String::new();
    let mut body = String::new();
    let mut active = 1; 

    loop {
        queue!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        queue!(
            stdout,
            SetForegroundColor(CrossColor::DarkGrey),
            Print("─".repeat(80)),
            Print("\n\r"),
            ResetColor,
            SetAttribute(Attribute::Bold),
            Print("  STRUCTURED COMMIT EDITOR"),
            SetAttribute(Attribute::Reset),
            Print("\n\r"),
            SetForegroundColor(CrossColor::DarkGrey),
            Print("─".repeat(80)),
            Print("\n\r\n\r"),
            ResetColor
        )?;

        let fields = [
            ("SUBMIT", ""),
            ("SCOPE", "Optional context"),
            ("SUBJECT", "Brief summary"),
            ("BODY", "Detailed motivation"),
        ];

        for i in 1..4 {
            let is_active = active == i;
            let label = fields[i].0;
            let hint = fields[i].1;
            let val = match i {
                1 => &scope,
                2 => &subject,
                3 => &body,
                _ => "",
            };

            if is_active {
                queue!(stdout, SetForegroundColor(CrossColor::Cyan), Print(" ● "), ResetColor)?;
            } else {
                queue!(stdout, Print("   "))?;
            }

            queue!(
                stdout,
                SetAttribute(Attribute::Bold),
                Print(format!("{:<10} ", label)),
                SetAttribute(Attribute::Reset),
                SetForegroundColor(CrossColor::DarkGrey),
                Print("│ "),
                ResetColor
            )?;

            if i == 3 {
                let lines: Vec<&str> = val.split('\n').collect();
                for (idx, line) in lines.iter().enumerate() {
                    if idx > 0 {
                        queue!(stdout, Print("\n\r             "), SetForegroundColor(CrossColor::DarkGrey), Print("│ "), ResetColor)?;
                    }
                    if is_active && idx == lines.len() - 1 {
                        queue!(stdout, SetForegroundColor(CrossColor::White), Print(line), Print("█"), ResetColor)?;
                    } else {
                        queue!(stdout, Print(line))?;
                    }
                }
                if is_active {
                    queue!(stdout, Print("\n\r             "), SetForegroundColor(CrossColor::DarkGrey), Print("└─ "), Print(hint), ResetColor)?;
                }
            } else {
                if is_active {
                    queue!(stdout, SetForegroundColor(CrossColor::White), Print(val), Print("█"), ResetColor, Print("  "), SetForegroundColor(CrossColor::DarkGrey), Print("// "), Print(hint), ResetColor)?;
                } else {
                    queue!(stdout, Print(val))?;
                }
            }
            queue!(stdout, Print("\n\r\n\r"))?;
        }

        queue!(stdout, Print("\n\r"))?;
        if active == 0 {
            queue!(stdout, SetForegroundColor(CrossColor::Green), SetAttribute(Attribute::Reverse), Print("    [ CONFIRM AND COMMIT ]    "), SetAttribute(Attribute::Reset), ResetColor)?;
        } else {
            queue!(stdout, SetForegroundColor(CrossColor::DarkGrey), Print("    [ CONFIRM AND COMMIT ]    "), ResetColor)?;
        }
        queue!(stdout, Print("\n\r\n\r"))?;

        queue!(
            stdout,
            SetForegroundColor(CrossColor::DarkGrey),
            Print("─".repeat(80)),
            Print("\n\r"),
            Print("  ↑/↓: Navigate • Enter: Next Field (Body: Newline) • Esc: Abort"),
            Print("\n\r"),
            Print("─".repeat(80)),
            ResetColor
        )?;

        stdout.flush()?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Up => {
                    active = match active {
                        1 => 0,
                        2 => 1,
                        3 => 2,
                        0 => 3,
                        _ => 1,
                    };
                }
                KeyCode::Down => {
                    active = match active {
                        1 => 2,
                        2 => 3,
                        3 => 0,
                        0 => 1,
                        _ => 1,
                    };
                }
                KeyCode::Enter => {
                    match active {
                        0 => break,
                        1 | 2 => active += 1,
                        3 => body.push('\n'),
                        _ => {}
                    }
                }
                KeyCode::Backspace => {
                    match active {
                        1 => { scope.pop(); }
                        2 => { subject.pop(); }
                        3 => { body.pop(); }
                        _ => {}
                    }
                }
                KeyCode::Char(c) => {
                    match active {
                        1 => scope.push(c),
                        2 => subject.push(c),
                        3 => body.push(c),
                        _ => {}
                    }
                }
                KeyCode::Esc => {
                    return Err(anyhow::anyhow!("Operation aborted."));
                }
                _ => {}
            }
        }
    }
    
    Ok((scope, subject, body))
}