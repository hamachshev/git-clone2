use commands::{add, cat_file, hash_object, status};
use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod index;
mod object;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Init a git repo
    Init,
    HashObject {
        #[arg(short)]
        write: bool,

        #[arg(long)]
        stdin: bool,

        file: Option<PathBuf>,
    },
    CatFile {
        #[arg(short)]
        pretty_print: bool,

        hash: String,
    },
    Add {
        file: PathBuf,
    },
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            //more to be implemnted
            fs::create_dir_all(".git/objects")?;
        }
        Commands::HashObject {
            write,
            stdin,
            ref file,
        } => {
            hash_object::invoke(file, write)?;
        }
        Commands::CatFile { pretty_print, hash } => {
            cat_file::invoke(*pretty_print, &hash)?;
        }
        Commands::Add { file } => add::invoke(file)?,
        Commands::Status => status::invoke()?,
    }
    Ok(())
}
