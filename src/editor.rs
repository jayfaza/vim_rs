#[derive(Debug)]
pub enum EditorMode {
    Normal,
    Insert,
    Command
}

#[derive(Debug)]
pub struct EditorState {
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub last_cursor: u16,
    pub cursor_line: usize,
    pub display_offset: usize,
    pub mode: EditorMode,
    pub command: String,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            last_cursor: 0,
            cursor_line: 0,
            display_offset: 0,
            mode: EditorMode::Normal,
            command: String::new(),
        }
    }
}
