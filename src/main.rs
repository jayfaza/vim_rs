mod args;
mod editor;

use std::io;

use clap::Parser;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use editor::Editor;
use args::Args;

fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut editor = Editor::new(&args.path);
    editor.update_terminal_size()?;
    enable_raw_mode();
    editor.editor_start()?;
    disable_raw_mode();

    Ok(())
}
