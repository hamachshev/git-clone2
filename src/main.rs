use commands::{cat_file, hash_object};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use object::{Kind, Object};
use std::{
    ffi::CStr,
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use sha1::{Digest, Sha1};

mod commands;
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
    }
    Ok(())
}
