mod cli;
mod utils;
mod update_loop;
mod screen;
mod editor;
mod file;
mod event_handler;

use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use clap::Parser;
use anyhow::bail;

use cli::Cli;
use utils::{is_dir, exists_or_create};
use file::OpenFile;
use screen::Screen;
use editor::EditorState;
use update_loop::UpdateLoop;


fn main() -> anyhow::Result<()> {
    let path = Cli::parse().path;
    if is_dir(&path) {
        bail!("Specefied path should be a file, not dir:\n{path}")
    }

    exists_or_create(&path)?;

    let mut file = OpenFile::new(&path)?;
    let mut screen = Screen::new()?;
    let mut editor_state = EditorState::new();
    enable_raw_mode()?;

    UpdateLoop::run(&mut screen, &mut file, &mut editor_state)?;
    disable_raw_mode()?;
    Ok(())
}


