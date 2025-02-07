use std::{fmt::Display, io::BufRead};

use anyhow::{Context, Result};

pub(crate) struct Commit {
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
    pub fn read(bufread: &mut impl BufRead) -> Result<Commit> {
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
