use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use object::Object;
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
                    let mut buffreader = BufReader::new(file);
                    let mut buffer = Vec::new();
                    let bytes = buffreader
                        .read_to_end(&mut buffer)
                        .context("reading file to buffer")?;

                    let file = fs::File::create_new("temp").context("wrting temp file")?;
                    let mut hash_writer = HashWriter {
                        hasher: Sha1::new(),
                        writer: ZlibEncoder::new(file, Compression::default()),
                    };

                    write!(hash_writer, "blob {}\0", bytes)
                        .context("writing  header to the file and hashing")?;
                    hash_writer
                        .write_all(&buffer)
                        .context("writing content to file and hashing")?;
                    let result = hash_writer.hasher.finalize();
                    let file = hash_writer
                        .writer
                        .finish()
                        .context("finishing the compression")?;

                    let hash = format!("{:x}", result);
                    fs::create_dir_all(format!(".git/objects/{}", &hash[..2]))
                        .context("creating the dir for the compressed file")?;
                    fs::rename(
                        "temp",
                        format!(".git/objects/{}/{}", &hash[..2], &hash[2..]),
                    )
                    .context("rename the file")?;

                    println!("{:x}", result)
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

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.hasher.update(buf);
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
