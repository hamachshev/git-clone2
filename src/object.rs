use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use std::{
    ffi::CStr,
    fs,
    io::{BufRead, BufReader, Read},
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
