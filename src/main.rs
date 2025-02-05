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
        write: Option<bool>,

        #[arg(long)]
        stdin: Option<bool>,

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
            if let Some(write) = write {
            } else {
                if let Some(file) = file {
                    let file =
                        fs::File::open(file).context("opening the file to read the contents")?;
                    let len = file.metadata().context("retrieving metadata")?.len();
                    let mut obj = Object {
                        kind: Kind::Blob,
                        reader: Box::new(file),
                        len,
                    };
                    let result = obj.write().context("writing file to objects")?;

                    println!("{}", result)
                }
            }
        }
        Commands::CatFile { pretty_print, hash } => {
            anyhow::ensure!(pretty_print, "must have pretty print for now");

            let mut obj: Object = hash.as_str().try_into().context("parsing object")?;
            match obj.kind {
                object::Kind::Blob => {
                    let mut buffer = Vec::new();
                    obj.reader.read_to_end(&mut buffer)?;
                    let contents = String::from_utf8_lossy(&buffer);

                    print!("{}", &contents);
                }
                object::Kind::Tree => todo!(),
                object::Kind::Commit => todo!(),
            }
        }
    }
    Ok(())
}
