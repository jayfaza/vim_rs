use crossterm::event::{self, poll, read};
use crossterm::style::Print;
use crossterm::{cursor, queue, terminal};
use std::io::{Stdout, Write, stdout};
use std::time::Duration;

use crate::editor::{EditorMode, EditorState};
use crate::event_handler::handle_key;
use crate::file::OpenFile;
use crate::screen::Screen;

pub struct UpdateLoop {}

impl UpdateLoop {
    pub fn run(
        screen: &mut Screen,
        file: &mut OpenFile,
        editor_state: &mut EditorState,
    ) -> anyhow::Result<()> {
        let mut stdout = stdout();
        redisplay(screen, file, editor_state, &mut stdout)?;
        loop {
            if poll(Duration::from_millis(50))?
                && let event::Event::Key(event) = read()?
            {
                handle_key(screen, editor_state, file, &mut stdout, event.code)?;
            }
        }
        Ok(())
    }
}

pub fn redisplay(
    screen: &mut Screen,
    file: &mut OpenFile,
    editor_state: &mut EditorState,
    stdout: &mut Stdout,
) -> anyhow::Result<()> {
    queue!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    for offset in 0..screen.h {
        if let Some(line) = file
            .content
            .get(editor_state.display_offset + offset as usize)
            && !line.is_empty()
        {
            queue!(stdout, cursor::MoveTo(0, offset), Print(line))?;
        }
    }

    if let EditorMode::Command = editor_state.mode {
        queue!(
            stdout,
            cursor::MoveTo(0, screen.h - 1),
            terminal::Clear(terminal::ClearType::CurrentLine),
            Print(":"),
            Print(&editor_state.command)
        )?;
        stdout.flush()?;
    }

    queue!(
        stdout,
        cursor::MoveTo(
            screen.w
                - format!("{}/{}", editor_state.cursor_line, editor_state.cursor_x).len() as u16,
            screen.h
        ),
        Print(format!(
            "{}/{}",
            editor_state.cursor_line, editor_state.cursor_x
        ))
    )?;

    queue!(
        stdout,
        cursor::MoveTo(editor_state.cursor_x, editor_state.cursor_y)
    )?;

    stdout.flush()?;

    Ok(())
}
