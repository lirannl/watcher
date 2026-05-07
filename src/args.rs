use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub folder: PathBuf,
    #[arg(short = 'a', long)]
    pub on_add: Option<String>,
    #[arg(short = 'r', long)]
    pub on_remove: Option<String>,
    #[arg(short = 'm', long)]
    pub on_modify: Option<String>,
}
