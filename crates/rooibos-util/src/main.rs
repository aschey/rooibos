use std::io::{self};
use std::path::PathBuf;

use clap::{Args, Parser};
use rooibos_util::{EmbedOrPath, parse_theme_css, read_themes_from_path};

mod generate_theme;
mod open_theme;
mod print_theme;

#[derive(Args)]
struct GenerateArgs {
    #[arg(long)]
    crate_dir: PathBuf,
    #[arg(long)]
    src_dir: PathBuf,
    #[arg(long)]
    dest_dir: PathBuf,
}

#[derive(Parser)]
enum Action {
    Generate(GenerateArgs),
    Open { theme_path: Option<PathBuf> },
    Print { theme_path: Option<PathBuf> },
}

fn main() -> io::Result<()> {
    let action = Action::parse();

    match action {
        Action::Generate(GenerateArgs {
            crate_dir,
            src_dir,
            dest_dir,
        }) => generate_theme::generate(&crate_dir, &EmbedOrPath::Path(src_dir), &dest_dir),
        Action::Open { theme_path } => {
            open_theme::open(&EmbedOrPath::from_optional_path(theme_path))
        }
        Action::Print { theme_path } => {
            print_theme::print(&EmbedOrPath::from_optional_path(theme_path))
        }
    }
}
