use anyhow::{Context, Result};
use std::{fmt::Display, io::BufRead};

pub(crate) struct TreeEntry {
    mode: String,
    filename: String,
    hash: String,
}
impl TreeEntry {
    pub fn read(bufread: &mut impl BufRead) -> Result<TreeEntry> {
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
