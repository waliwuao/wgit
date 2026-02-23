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
    let mut active = 0; 

    loop {
        queue!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        queue!(
            stdout,
            Print("=== Commit Form ===\n\r"),
            Print("Use [Up/Down] to switch fields. Select < SUBMIT > and press [Enter] to commit.\n\r"),
            Print("Press [Esc] to Abort.\n\r\n\r")
        )?;

        let labels = ["< SUBMIT >", "Scope (opt): ", "Subject:     ", "Body (opt):  "];
        let values = ["", &scope, &subject, &body];

        for i in 0..4 {
            if i == active {
                queue!(stdout, SetForegroundColor(CrossColor::Green), Print("> "), ResetColor)?;
                if i == 0 {
                    queue!(stdout, SetAttribute(Attribute::Reverse), Print(labels[i]), SetAttribute(Attribute::Reset))?;
                } else {
                    queue!(stdout, Print(labels[i]))?;
                }
            } else {
                queue!(stdout, Print("  "), Print(labels[i]))?;
            }

            if i > 0 {
                if i == 3 { 
                    // Body handling
                    let lines: Vec<&str> = values[i].split('\n').collect();
                    for (j, line) in lines.iter().enumerate() {
                        if j > 0 {
                            queue!(stdout, Print("\n\r"), Print("               "))?;
                        }
                        queue!(stdout, Print(line))?;
                    }
                    queue!(stdout, Print("\n\r"))?;
                } else {
                    queue!(stdout, Print(values[i]), Print("\n\r"))?;
                }
            } else {
                queue!(stdout, Print("\n\r"))?;
            }
        }
        stdout.flush()?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Up => {
                    if active > 0 { active -= 1; }
                }
                KeyCode::Down => {
                    if active < 3 { active += 1; }
                }
                KeyCode::Enter => {
                    if active == 0 {
                        // Submit
                        break;
                    } else if active == 3 {
                        body.push('\n');
                    } else { 
                        active += 1; 
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
                    return Err(anyhow::anyhow!("Commit form aborted by user."));
                }
                _ => {}
            }
        }
    }
    
    Ok((scope, subject, body))
}