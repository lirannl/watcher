use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub folder: PathBuf,
    pub command: Option<String>,
}
