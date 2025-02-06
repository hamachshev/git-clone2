use anyhow::Result;
use std::fs;

use crate::index::IndexEntry;
pub fn invoke(file: &std::path::PathBuf) -> Result<()> {
    if fs::exists(".git/index").is_err() {
        fs::File::create_new(".git/index");
    }

    let index_entry = IndexEntry::from_path("Cargo.toml".into(), "sdlkajjlks", 0)?;
    println!("{:#?}", index_entry);
    Ok(())
}
