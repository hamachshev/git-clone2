use std::{
    fmt::Display,
    io::{BufRead, BufReader, Read},
};

use crate::object::{self, Object};
use anyhow::{Context, Result};
use flate2::bufread;
pub fn invoke(pretty_print: bool, hash: &str) -> Result<()> {
    anyhow::ensure!(pretty_print, "must have pretty print for now");

    let mut obj: Object = hash.try_into().context("parsing object")?;
    match obj.kind {
        object::Kind::Blob => {
            let mut buffer = Vec::new();
            obj.reader.read_to_end(&mut buffer)?;
            let contents = String::from_utf8_lossy(&buffer);

            print!("{}", &contents);
        }
        object::Kind::Tree => {
            let mut bufread = BufReader::new(&mut obj.reader);
            while let Ok(tree_entry) = TreeEntry::read(&mut bufread) {
                print!("{}", tree_entry);
            }
        }
        object::Kind::Commit => todo!(),
    }

    Ok(())
}

struct TreeEntry {
    mode: String,
    filename: String,
    hash: String,
}
impl TreeEntry {
    fn read(bufread: &mut impl BufRead) -> Result<TreeEntry> {
        let mut mode_and_filename = Vec::new();
        let bytes_read = bufread
            .read_until(0, &mut mode_and_filename)
            .context("reading mode and file name")?;
        anyhow::ensure!(bytes_read > 1, "end of file");

        let mode_and_filename =
            String::from_utf8_lossy(&mode_and_filename[..mode_and_filename.len() - 1]);
        let (mode, filename) = mode_and_filename
            .split_once(' ')
            .context("splitting mode and filename")?;
        let mut hash = [0u8; 20];
        bufread.read_exact(&mut hash).context("reading hash")?;
        let hash = hex::encode(hash);

        let tree_entry = TreeEntry {
            mode: mode.to_string(),
            filename: filename.to_string(),
            hash,
        };

        Ok(tree_entry)
    }
}

impl Display for TreeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file_type = match self.mode.as_str() {
            "100644" => "blob",
            "100755" => "executable",
            "40000" => "tree",
            "120000" => "symlink",
            "160000" => "gitlink",
            _ => "",
        };
        writeln!(
            f,
            "{:06} {} {} {}",
            self.mode, file_type, self.hash, self.filename
        )
    }
}
