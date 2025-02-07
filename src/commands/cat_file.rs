use crate::object::{self, Object};
use anyhow::{Context, Result};
use std::{
    fmt::Display,
    io::{BufRead, BufReader, Read},
};

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
        object::Kind::Commit => {
            let mut bufread = BufReader::new(&mut obj.reader);
            let commit = Commit::read(&mut bufread)?;
            print!("{}", &commit);
        }
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

struct Commit {
    tree: String,
    parent: String,
    author: String,
    author_email: String,
    author_time: String,
    author_offset: String,
    committer: String,
    committer_email: String,
    committer_time: String,
    committer_offset: String,
    message: String,
}

impl Commit {
    fn read(bufread: &mut impl BufRead) -> Result<Commit> {
        let mut tree_hash = String::new();
        bufread.read_line(&mut tree_hash)?;
        let (_, tree_hash) = tree_hash
            .split_once(' ')
            .context("error in formatting in tree hash")?;

        let mut parent_hash = String::new();
        bufread.read_line(&mut parent_hash)?;
        let (_, parent_hash) = parent_hash
            .split_once(' ')
            .context("error in formatting in parent hash")?;

        let mut author = String::new();
        bufread.read_line(&mut author)?;
        let author_vec: Vec<_> = author.split_whitespace().collect();
        anyhow::ensure!(author_vec.len() == 6, "author entry wrong format");

        let mut committer = String::new();
        bufread.read_line(&mut committer)?;
        let committer_vec: Vec<_> = committer.split_whitespace().collect();
        anyhow::ensure!(committer_vec.len() == 6, "committer entry wrong format");

        let mut message = String::new();
        bufread.read_line(&mut message)?;
        message.clear();
        bufread.read_line(&mut message)?;

        Ok(Commit {
            tree: tree_hash.trim_end().to_string(),
            parent: parent_hash.trim_end().to_string(),
            author: format!("{} {}", &author_vec[1], &author_vec[2]),
            author_email: author_vec[3].to_string(),
            author_time: author_vec[4].to_string(),
            author_offset: author_vec[5].trim_end().to_string(),
            committer: format!("{} {}", &committer_vec[1], &committer_vec[2]),
            committer_email: committer_vec[3].to_string(),
            committer_time: committer_vec[4].to_string(),
            committer_offset: committer_vec[5].trim_end().to_string(),
            message,
        })
    }
}

impl Display for Commit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "tree {}", self.tree)?;
        writeln!(f, "parent {}", self.parent)?;
        writeln!(
            f,
            "author {} {} {} {}",
            self.author, self.author_email, self.author_time, self.author_offset
        )?;
        writeln!(
            f,
            "committer {} {} {} {}\n",
            self.committer, self.committer_email, self.committer_time, self.committer_offset
        )?;
        write!(f, "{}", self.message)?;
        Ok(())
    }
}
