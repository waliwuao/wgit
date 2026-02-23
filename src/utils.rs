use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    queue,
    style::{Color as CrossColor, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{stdout, Write};

pub fn get_theme() -> inquire::ui::RenderConfig<'static> {
    use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};

    let selected_style = StyleSheet::new().with_fg(Color::LightBlue);
    
    RenderConfig::default()
        .with_selected_option(Some(selected_style))
        .with_selected_checkbox(Styled::new("☑").with_fg(Color::LightBlue))
        .with_scroll_up_prefix(Styled::new("▲").with_fg(Color::LightBlue))
        .with_scroll_down_prefix(Styled::new("▼").with_fg(Color::LightBlue))
        .with_prompt_prefix(Styled::new("?").with_fg(Color::LightGreen))
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
    let mut active = 0; 

    loop {
        queue!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        queue!(
            stdout,
            Print("=== Commit Form ===\n\r"),
            Print("Use [Up/Down] to switch fields, [Enter] to add new line in Body.\n\r"),
            Print("Press [Ctrl+Enter] or [Ctrl+S] to Submit, [Esc] to Abort.\n\r\n\r")
        )?;

        let fields = ["Scope:   ", "Subject: ", "Body:    "];
        let values = [&scope, &subject, &body];

        for i in 0..3 {
            if i == active {
                queue!(stdout, SetForegroundColor(CrossColor::Green), Print("> "), ResetColor)?;
            } else {
                queue!(stdout, Print("  "))?;
            }
            queue!(stdout, Print(fields[i]))?;

            if i == 2 {
                let lines: Vec<&str> = values[i].split('\n').collect();
                for (j, line) in lines.iter().enumerate() {
                    if j > 0 {
                        queue!(stdout, Print("\n\r"), Print("           "))?;
                    }
                    queue!(stdout, Print(line))?;
                }
                queue!(stdout, Print("\n\r"))?;
            } else {
                queue!(stdout, Print(values[i]), Print("\n\r"))?;
            }
        }
        stdout.flush()?;

        if let Event::Key(key_event) = event::read()? {
            let is_submit = key_event.modifiers.contains(KeyModifiers::CONTROL) &&
                (key_event.code == KeyCode::Enter || 
                 key_event.code == KeyCode::Char('j') || 
                 key_event.code == KeyCode::Char('m') || 
                 key_event.code == KeyCode::Char('s'));

            if is_submit {
                break;
            }

            match key_event.code {
                KeyCode::Up => {
                    if active > 0 { active -= 1; }
                }
                KeyCode::Down => {
                    if active < 2 { active += 1; }
                }
                KeyCode::Enter => {
                    if active == 2 {
                        body.push('\n');
                    } else if active < 2 { 
                        active += 1; 
                    }
                }
                KeyCode::Backspace => {
                    match active {
                        0 => { scope.pop(); }
                        1 => { subject.pop(); }
                        2 => { body.pop(); }
                        _ => {}
                    }
                }
                KeyCode::Char(c) => {
                    match active {
                        0 => scope.push(c),
                        1 => subject.push(c),
                        2 => body.push(c),
                        _ => {}
                    }
                }
                KeyCode::Esc => {
                    return Err(anyhow::anyhow!("Commit form aborted by user."));
                }
                _ => {}
            }
        }
    }
    
    Ok((scope, subject, body))
}