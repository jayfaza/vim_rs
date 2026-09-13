use crossterm::event::KeyCode::{self, Backspace, Char, Enter, Esc};
use std::{io::Stdout, process::exit};

use crate::{
    editor::{EditorMode, EditorState},
    file::OpenFile,
    screen::Screen,
    update_loop::redisplay,
};

pub fn handle_key(
    screen: &mut Screen,
    editor_state: &mut EditorState,
    file: &mut OpenFile,
    stdout: &mut Stdout,
    key: KeyCode,
) -> anyhow::Result<()> {
    match editor_state.mode {
        EditorMode::Normal => match key {
            Char('j') => {
                key_j(file, editor_state, screen)?;
                redisplay(screen, file, editor_state, stdout)?;
            }
            Char('k') => {
                key_k(editor_state, file)?;
                redisplay(screen, file, editor_state, stdout)?;
            }
            Char('h') => {
                key_h(editor_state);
                redisplay(screen, file, editor_state, stdout)?;
            }
            Char('l') => {
                key_l(editor_state, file);
                redisplay(screen, file, editor_state, stdout)?;
            }
            Char('i') => {
                editor_state.mode = EditorMode::Insert;
            }
            Char(':') => {
                editor_state.mode = EditorMode::Command;
                redisplay(screen, file, editor_state, stdout)?;
            }
            Char('q') => exit(0),
            _ => {}
        },
        EditorMode::Insert => {
            if key == Esc {
                editor_state.mode = EditorMode::Normal;
                return Ok(());
            }
            event_insert(editor_state, file, key)?;
            redisplay(screen, file, editor_state, stdout)?;
        }
        EditorMode::Command => {
            match key {
                Esc => {
                    editor_state.mode = EditorMode::Normal;
                    editor_state.command.clear();
                    return Ok(());
                }
                Backspace => {
                    if !editor_state.command.is_empty() {
                        editor_state.command.pop();
                        redisplay(screen, file, editor_state, stdout)?;
                    }
                }
                Enter => {
                    match editor_state.command.as_str() {
                        "q" => exit(0),
                        "w" => file.write()?,
                        "wq" => {
                            file.write()?;
                            exit(0);
                        }
                        _ => {}
                    }
                    editor_state.command.clear();
                    editor_state.mode = EditorMode::Normal;
                    redisplay(screen, file, editor_state, stdout)?;
                }
                Char(ch) => {
                    editor_state.command.push(ch);
                    redisplay(screen, file, editor_state, stdout)?;
                }
                _ => {},
            }
        }
    }

    Ok(())
}

fn event_insert(
    editor_state: &mut EditorState,
    file: &mut OpenFile,
    event: KeyCode,
) -> anyhow::Result<()> {
    match event {
        KeyCode::Backspace => {
            backspace(editor_state, file)?;
        }
        Char(ch) => {
            if let Some(line) = file.content.get(editor_state.cursor_line) {
                let mut current_line: Vec<char> = line.clone().chars().collect();
                current_line.insert(editor_state.cursor_x as usize, ch);
                let mut result_line = String::new();
                current_line.into_iter().for_each(|ch| result_line.push(ch));
                editor_state.cursor_x += 1;
                file.content[editor_state.cursor_line] = result_line;
            }
        }
        _ => {}
    }
    Ok(())
}

fn backspace(editor_state: &mut EditorState, file: &mut OpenFile) -> anyhow::Result<()> {
    if let Some(cur_line) = file.content.get(editor_state.cursor_line) {
        let mut current_line: Vec<char> = cur_line.clone().chars().collect();
        if editor_state.cursor_x > 0 {
            current_line.remove(editor_state.cursor_x as usize - 1);
            editor_state.cursor_x -= 1;
        }
        let mut result_line = String::new();
        current_line.into_iter().for_each(|ch| result_line.push(ch));
        file.content[editor_state.cursor_line] = result_line;
    }
    Ok(())
}

fn key_j(
    file: &mut OpenFile,
    editor_state: &mut EditorState,
    screen: &mut Screen,
) -> anyhow::Result<()> {
    if editor_state.cursor_y < screen.h {
        if editor_state.cursor_line < file.length {
            if let Some(bottom_line) = file.content.get(editor_state.cursor_line + 1) {
                if bottom_line.len() < editor_state.cursor_x.into() {
                    editor_state.cursor_x = bottom_line.len() as u16;
                }
                if bottom_line.len() >= editor_state.last_cursor.into() {
                    editor_state.cursor_x = editor_state.last_cursor;
                }
                if bottom_line.len() > editor_state.cursor_x.into()
                    && editor_state.cursor_x < editor_state.last_cursor
                {
                    editor_state.cursor_x = bottom_line.len() as u16;
                }
            }
            editor_state.cursor_y += 1;
            editor_state.cursor_line += 1;
        }
    } else {
        if editor_state.display_offset < file.length && editor_state.cursor_line < file.length {
            if let Some(bottom_line) = file.content.get(editor_state.cursor_line) {
                if bottom_line.len() < editor_state.cursor_x.into() {
                    editor_state.cursor_x = bottom_line.len() as u16;
                }
                if bottom_line.len() >= editor_state.last_cursor.into() {
                    editor_state.cursor_x = editor_state.last_cursor;
                }
                if bottom_line.len() > editor_state.cursor_x.into()
                    && editor_state.cursor_x < editor_state.last_cursor
                {
                    editor_state.cursor_x = bottom_line.len() as u16;
                }
            }
            editor_state.display_offset += 1;
            editor_state.cursor_line += 1;
        }
    }
    Ok(())
}

fn key_k(editor_state: &mut EditorState, file: &mut OpenFile) -> anyhow::Result<()> {
    if editor_state.cursor_y > 0 {
        if editor_state.cursor_line > 0
            && let Some(top_line) = file.content.get(editor_state.cursor_line - 1)
        {
            if top_line.len() < editor_state.cursor_x as usize {
                editor_state.cursor_x = top_line.len() as u16;
            }
            if top_line.len() >= editor_state.last_cursor.into() {
                editor_state.cursor_x = editor_state.last_cursor;
            }
            if top_line.len() > editor_state.cursor_x.into()
                && editor_state.cursor_x < editor_state.last_cursor
            {
                editor_state.cursor_x = top_line.len() as u16;
            }
            if top_line.is_empty() {
                editor_state.cursor_x = 0;
            }
        }
        editor_state.cursor_y -= 1;
        editor_state.cursor_line -= 1;
    } else {
        if editor_state.cursor_line > 0
            && let Some(top_line) = file.content.get(editor_state.cursor_line - 1)
        {
            if top_line.len() < editor_state.cursor_x as usize {
                editor_state.cursor_x = top_line.len() as u16;
            }
            if top_line.len() >= editor_state.last_cursor.into() {
                editor_state.cursor_x = editor_state.last_cursor;
            }
            if top_line.len() > editor_state.cursor_x.into()
                && editor_state.cursor_x < editor_state.last_cursor
            {
                editor_state.cursor_x = top_line.len() as u16;
            }
            if top_line.is_empty() {
                editor_state.cursor_x = 0;
            }
        }
        if editor_state.display_offset > 0 {
            editor_state.display_offset -= 1;
            editor_state.cursor_line -= 1;
        }
    }
    Ok(())
}

fn key_l(editor_state: &mut EditorState, file: &mut OpenFile) {
    if let Some(line) = file.content.get(editor_state.cursor_line) {
        let cur_line_len = line.len();
        if editor_state.cursor_x < cur_line_len as u16 {
            editor_state.cursor_x += 1;
            editor_state.last_cursor = editor_state.cursor_x;
        }
    }
}

fn key_h(editor_state: &mut EditorState) {
    if editor_state.cursor_x > 0 {
        editor_state.cursor_x -= 1;
        editor_state.last_cursor = editor_state.cursor_x;
    }
}
