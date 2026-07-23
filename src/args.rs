use clap::Parser;
use anyhow::{bail, Ok, Result};
use std::path::Path;
use shellexpand;

#[derive(Parser)]
pub struct Args {
    #[arg(short = 'p', long = "path")]
    pub path: String,
}

impl Args {
    pub fn process_path(&mut self) -> Result<Args> {
        let processed_path = shellexpand::tilde(&self.path).to_string();
        let os_path = Path::new(&processed_path);
        dbg!(os_path);
        if !os_path.is_dir() {
            let mut args = Args::parse();
            args.path = processed_path.to_string();
            return Ok(args);
        } else {
            bail!("Specified path have to be a file.")
        }
    }
}
