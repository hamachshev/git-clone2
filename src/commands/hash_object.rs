use std::{fs, path::PathBuf};

use anyhow::{Context, Result};

use crate::object::{Kind, Object};

pub fn invoke(file: &Option<PathBuf>, write: &bool) -> Result<()> {
    if let Some(file) = file {
        let file = fs::File::open(file).context("opening the file to read the contents")?;
        let len = file.metadata().context("retrieving metadata")?.len();
        let mut obj = Object {
            kind: Kind::Blob,
            reader: Box::new(file),
            len,
        };
        if *write {
            let result = obj.write().context("writing file to objects")?;

            println!("{}", result);
            Ok(())
        } else {
            let result = obj.hash().context("computing hash")?;
            println!("{}", result);
            Ok(())
        }
    } else {
        anyhow::bail!("need to input file");
    }
}
