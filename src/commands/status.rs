use std::{
    collections::HashSet,
    fs,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::objects::{
    commit::Commit,
    index::{IndexEntry, IndexFile},
    object::Object,
    tree::Tree,
};

pub fn invoke() -> Result<()> {
    let mut head_file = fs::File::open(".git/HEAD").context("opening HEAD file")?;
    let mut head_ref = String::new();

    head_file
        .read_to_string(&mut head_ref)
        .context("reading HEAD file")?;

    let head_hash = if let Some((_, head_ref)) = head_ref.split_once(" ") {
        let head_ref = format!(".git/{}", &head_ref);

        let mut head_hash_file =
            fs::File::open(head_ref.as_str().trim()).context("opening head_hash file")?;

        let mut head_hash = String::new();
        head_hash_file
            .read_to_string(&mut head_hash)
            .context("reading head hash file")?;

        head_hash.trim().to_string()
    } else {
        // we are in detached state where the HEAD file stores a hash
        head_ref.trim().to_string()
    };

    let commit = Commit::read_from_hash(&head_hash)?;

    let mut files_in_commit = HashSet::new();

    let mut tree_object =
        Object::try_from(commit.tree.as_str()).context("reading tree object from commit")?;
    let mut bufread = BufReader::new(&mut tree_object.reader);
    let tree = Tree::read(&mut bufread).context("reading tree from tree object from commit")?;
    tree.traverse(&mut files_in_commit)
        .context("getting files_in_commit of blobs in commit")?;

    let mut modified_files = Vec::new();
    let mut staged_files = Vec::new();

    let index_file = IndexFile::read_from_index()?;

    for entry in index_file.entries.iter() {
        let other = IndexEntry::from_path(entry.entry_path.clone(), &entry.hash, entry.flags)
            .context("constructing index entry from path")?;
        if other != *entry {
            modified_files.push(entry.entry_path.clone());
        }

        if !files_in_commit.contains(entry.hash.as_str()) {
            //ie if files_in_commit does not contain one of the index files, it is new, ie staged
            //for commit
            staged_files.push(entry.entry_path.clone());
        }
    }

    let index_hash_set: HashSet<String> = index_file
        .entries
        .into_iter()
        .map(|e| {
            let path_str: String = e.entry_path.clone().to_str().unwrap().parse().unwrap();
            path_str
        })
        .collect();

    let mut unstaged = Vec::new();
    check_dir_for_unstaged(Path::new("."), &index_hash_set, &mut unstaged)?;

    println!("modified----------------------------------");
    for file in modified_files {
        println!("{:#?}", file);
    }
    println!("staged-------------------------------------");
    for file in staged_files {
        println!("{:#?}", file);
    }
    println!("unstaged-------------------------------------");
    for file in unstaged {
        println!("{:#?}", file);
    }

    Ok(())
}

fn check_dir_for_unstaged(
    dir: &Path,
    index_hash_set: &HashSet<String>,
    mut unstaged: &mut Vec<PathBuf>,
) -> Result<()> {
    anyhow::ensure!(dir.is_dir(), "must pass dir for this to work");

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && !(path.starts_with("./target") || path.starts_with("./.git")) {
            check_dir_for_unstaged(&path, &index_hash_set, &mut unstaged)?;
        } else {
            let path_trunc = path.to_str().context("unlikely error in path trunc")?;
            if !index_hash_set.contains(&path_trunc[2..]) {
                unstaged.push(path);
            }
        }
    }
    Ok(())
}
