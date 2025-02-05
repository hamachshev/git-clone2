use crate::object::{self, Object};
use anyhow::{Context, Result};
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
        object::Kind::Tree => todo!(),
        object::Kind::Commit => todo!(),
    }

    Ok(())
}
