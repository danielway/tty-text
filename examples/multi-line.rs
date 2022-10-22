use std::io::{stdout, Stdout, Write};

use crossterm::{
    cursor,
    event::{self, KeyCode, KeyModifiers},
    queue,
    style::Print,
    terminal::{self, enable_raw_mode},
    Result,
};
use tty_text::{Key, Text};

/// A simple, multi-line CLI text editor built with crossterm.
fn main() {
    execute().expect("execute basic multi-line example");
}

fn execute() -> Result<()> {
    let mut stdout = stdout();
    let mut text = Text::new(true);

    enable_raw_mode()?;
    render(&mut stdout, &text)?;

    loop {
        let event = event::read()?;

        if let event::Event::Key(key_event) = event {
            if key_event.modifiers == KeyModifiers::CONTROL && key_event.code == KeyCode::Char('c')
            {
                break;
            }

            match key_event.code {
                KeyCode::Esc => break,
                KeyCode::Char(ch) => text.handle_input(Key::Char(ch)),
                KeyCode::Backspace => text.handle_input(Key::Backspace),
                KeyCode::Enter => text.handle_input(Key::Enter),
                KeyCode::Up => text.handle_input(Key::Up),
                KeyCode::Down => text.handle_input(Key::Down),
                KeyCode::Left => text.handle_input(Key::Left),
                KeyCode::Right => text.handle_input(Key::Right),
                _ => {}
            }
        }

        render(&mut stdout, &text)?;
    }

    Ok(())
}

fn render(stdout: &mut Stdout, text: &Text) -> Result<()> {
    queue!(stdout, terminal::Clear(terminal::ClearType::All))?;

    queue!(stdout, cursor::MoveTo(0, 0))?;
    queue!(
        stdout,
        Print("Enter text (arrows to move cursor, Ctrl/Cmd+C or Esc to quit):")
    )?;

    let lines = text.lines().iter().enumerate();
    for (line_index, line) in lines {
        queue!(stdout, cursor::MoveTo(0, line_index as u16 + 1))?;
        queue!(stdout, Print(line))?;
    }

    let position = text.cursor();
    let (x, y) = (position.0 as u16, position.1 as u16);
    queue!(stdout, cursor::MoveTo(x, y + 1))?;

    stdout.flush()?;

    Ok(())
}
