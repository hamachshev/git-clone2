use anyhow::{Context, Result};

use crate::objects::index::{IndexEntry, IndexFile};

pub fn invoke() -> Result<()> {
    let index_file = IndexFile::read_from_index()?;
    let changed_files = index_file.entries.iter().filter_map(|e| {
        let other = IndexEntry::from_path(e.entry_path.clone(), &e.hash, e.flags)
            .context("constructing index entry from path")
            .ok()?;
        if other == *e {
            None
        } else {
            Some(e.entry_path.clone())
        }
    });
    for item in changed_files {
        println!("{:#?}", item);
    }
    Ok(())
}
