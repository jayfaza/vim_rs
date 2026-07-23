mod args;
mod editor;

use clap::Parser;
use editor::Editor;
use args::Args;
use anyhow::Result;

fn main() -> Result<()> {
    let args = Args::parse().process_path()?;
    let mut editor = Editor::new(&args.path);

    editor.update_terminal_size()?;
    editor.editor_start()?;

    Ok(())
}
