use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(short = 'p', long = "path")]
    pub path: String,
}
