mod args;
mod editor;

use anyhow;
use args::Args;
use clap::Parser;
use editor::Editor;

fn main() -> anyhow::Result<()> {
    let args = Args::parse().process_path()?;
    let mut editor = Editor::new(&args.path)?;

    editor.update_terminal_size()?;
    editor.editor_start()?;

    Ok(())
}
