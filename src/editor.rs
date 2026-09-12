#[derive(Debug)]
pub struct EditorState {
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub cursor_line: usize,
    pub display_offset: usize,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            cursor_line: 0,
            display_offset: 0,
        }
    }
}
