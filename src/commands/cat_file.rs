use crate::objects::{
    commit::Commit,
    object::{self, Object},
    tree::Tree,
};
use anyhow::{Context, Result};
use std::io::{BufReader, Read};

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
            let tree = Tree::read(&mut bufread)?;

            for entry in tree.entries {
                println!("{}", entry);
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
