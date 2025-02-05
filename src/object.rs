use anyhow::{Context, Result};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{
    ffi::CStr,
    fs,
    io::{BufRead, BufReader, Read, Write},
};

pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
}

impl TryFrom<&str> for Kind {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "blob" => Ok(Kind::Blob),
            "tree" => Ok(Kind::Tree),
            "commit" => Ok(Kind::Commit),
            _ => anyhow::bail!("unknown object type"),
        }
    }
}

pub(crate) struct Object {
    pub kind: Kind,
    pub len: u64,
    pub reader: Box<dyn Read>, // must be dyn read because tryfrom must return an instance of Object,
                               // but if Object is generic, then tryfrom is defined for a specific R:Read, but depending on
                               // runtime logic, it will return a differnt reader.  see similar
                               // https://stackoverflow.com/questions/73876408/type-mismatch-expected-type-parameter-n-found-struct-vecu8
}

impl TryFrom<&str> for Object {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut path_iter = fs::read_dir(format!(".git/objects/{}", &value[..2]))?
            .into_iter()
            .filter(|e| {
                // get back result from the read_dir so have to unwrap
                let Ok(e) = e else {
                    return false;
                };
                let file_name = e.file_name();
                //have to convert to str to use starts with, ok bc must be valid utf-8 for git
                //anyway
                let Some(file_name) = file_name.to_str() else {
                    return false;
                };
                file_name.starts_with(&value[2..])
            });

        // double ?? because read_dir returns a result, and .next() returns an option so you get
        // Result<Option<>>
        let path = path_iter.next().context("error in getting path")??;
        anyhow::ensure!(path_iter.next().is_none(), "not a unique hash");

        let file = fs::File::open(path.path()).context("opening the file to read the contents")?;
        let z = ZlibDecoder::new(file);
        let mut buf_read = BufReader::new(z);
        let mut header = Vec::new();
        buf_read
            .read_until(0u8, &mut header)
            .context("reading header")?;
        let header = CStr::from_bytes_with_nul(&header).expect("must be a nul byte at the end");
        let header = header.to_str().context("header must be valid utf-8")?;
        let Some((blob_type, size)) = header.split_once(" ") else {
            anyhow::bail!("wrong format of file");
        };

        let size: u64 = size.parse().context("invalid size")?;

        Ok(Object {
            kind: blob_type.trim().try_into().context("error parsing kind")?,
            len: size,
            reader: Box::new(buf_read.take(size)),
        })
    }
}

impl Object {
    pub fn write(&mut self) -> Result<String> {
        let mut buffer = Vec::new();
        let bytes = self
            .reader
            .read_to_end(&mut buffer)
            .context("reading file to buffer")?;

        let file = fs::File::create_new("temp").context("wrting temp file")?;
        let mut hash_writer = HashWriter {
            hasher: Sha1::new(),
            writer: ZlibEncoder::new(file, Compression::default()),
        };

        match self.kind {
            Kind::Blob => {
                write!(hash_writer, "blob {}\0", bytes)
                    .context("writing  header to the file and hashing")?;
                hash_writer
                    .write_all(&buffer)
                    .context("writing content to file and hashing")?;
            }
            Kind::Tree => todo!(),
            Kind::Commit => todo!(),
        }

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

        Ok(hash)
    }
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
