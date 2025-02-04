use std::{
    fs,
    io::{BufReader, Read, Write},
    path::PathBuf,
};

use anyhow::Result;
use clap::{Parser, Subcommand};

use hex_literal::hex;
use sha1::{Digest, Sha1};

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
        write: Option<bool>,

        #[arg(long)]
        stdin: Option<bool>,

        file: Option<PathBuf>,
    },
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
            if let Some(write) = write {
            } else {
                if let Some(file) = file {
                    let file = fs::File::open(file)?;
                    let mut buffreader = BufReader::new(file);
                    let mut buffer = Vec::new();
                    let bytes = buffreader.read_to_end(&mut buffer)?;

                    let mut hasher = Sha1::new();
                    let header = format!("blob {bytes}");
                    hasher.update(header.into_bytes());
                    hasher.update(b"\0");
                    hasher.update(&buffer);

                    let result = hasher.finalize();

                    println!("{:x}", result)
                }
            }
        }
    }
    Ok(())
}
