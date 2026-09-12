use crate::{editor::EditorState, file::OpenFile, screen::Screen};

pub enum EventKey {
    KeyJ,
    KeyK,
    KeyL,
    KeyH,
}

pub fn handle_key(
    screen: &mut Screen,
    editor_state: &mut EditorState,
    file: &mut OpenFile,
    key: EventKey,
) -> anyhow::Result<()> {
    match key {
        EventKey::KeyJ => key_j(file, editor_state, screen),
        EventKey::KeyK => key_k(editor_state, file),
        EventKey::KeyL => {
            key_l(editor_state, file);
            Ok(())
        }
        EventKey::KeyH => {
            key_h(editor_state);
            Ok(())
        }
    }
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
            && let Some(top_line) = file.content.get(editor_state.cursor_line - 1) {
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
            && let Some(top_line) = file.content.get(editor_state.cursor_line - 1) {
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
