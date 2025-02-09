use crate::objects::object::{Kind, Object};
use anyhow::{Context, Result};
use std::{
    collections::HashSet,
    fmt::Display,
    io::{BufRead, BufReader},
};

pub(crate) struct Tree {
    pub entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn read(bufread: &mut impl BufRead) -> Result<Tree> {
        let mut entries = Vec::new();
        while let Ok(entry) = TreeEntry::read(bufread) {
            entries.push(entry);
        }
        if entries.len() == 0 {
            anyhow::bail!("this has no tree entries")
        }
        Ok(Tree { entries })
    }
    pub fn traverse(self, blobs: &mut HashSet<String>) -> Result<()> {
        for entry in self.entries {
            match entry {
                entry if entry.mode != 40000 => {
                    blobs.insert(entry.hash.clone());
                }
                entry if entry.mode == 40000 => {
                    let mut tree: Object = entry.hash.as_str().try_into()?;
                    anyhow::ensure!(tree.kind == Kind::Tree, "error in formatting");
                    let mut bufread = BufReader::new(&mut tree.reader);
                    let tree = Tree::read(&mut bufread).context("creating tree")?;
                    tree.traverse(blobs)?;
                }
                _ => anyhow::bail!("this should never be called"),
            }
        }
        Ok(())
    }
}
pub(crate) struct TreeEntry {
    mode: u32,
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
            mode: mode.parse().context("mode not a number")?,
            filename: filename.to_string(),
            hash,
        };

        Ok(tree_entry)
    }
}

impl Display for TreeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file_type = match self.mode {
            100644 => "blob",
            100755 => "executable",
            40000 => "tree",
            120000 => "symlink",
            160000 => "gitlink",
            _ => "",
        };
        write!(
            f,
            "{:06} {} {} {}",
            self.mode, file_type, self.hash, self.filename
        )
    }
}
