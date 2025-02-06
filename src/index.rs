use std::os::macos::fs::MetadataExt;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::PermissionsExt;
use std::{
    ffi::OsString,
    fs,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
};

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct IndexFile {
    pub signature: String,
    pub version: u32,
    pub entries: Vec<IndexEntry>,
}
impl IndexFile {
    pub fn read_from_index() -> Result<IndexFile> {
        let index = fs::File::open(".git/index").context("opening index file")?;
        let mut bufread = BufReader::new(index);
        let mut signature = [0u8; 4];
        bufread.read_exact(&mut signature)?;
        let signature = String::from_utf8_lossy(&signature);
        println!("{}", &signature);

        anyhow::ensure!(signature == "DIRC", "wrong signature, should be D I R C");

        let version = read_u32(&mut bufread).context("reading version")?;
        println!("{}", &version);

        let entries_number = read_u32(&mut bufread).context("reading version")?;
        println!("{}", &entries_number);

        let mut entries = Vec::new();
        for _ in 0..entries_number {
            let entry = IndexEntry::read_from_index(&mut bufread)?;
            entries.push(entry);
        }

        Ok(IndexFile {
            signature: signature.to_string(),
            version,
            entries,
        })
    }
}

#[derive(Debug)]
pub(crate) struct IndexEntry {
    //https://git-scm.com/docs/index-format
    pub ctime_seconds: u32,
    pub ctime_nanoseconds: u32,
    pub mtime_seconds: u32,
    pub mtime_nanoseconds: u32,
    pub device: u32,
    pub inode: u32,
    pub object_type: u8,      // 4bit valid types are in binary 1000, 1010, 1110
    pub unix_permission: u16, // 9 bit valid types 0775 and 0644 in octal
    // permission
    pub user_id: u32,
    pub group_id: u32,
    pub file_size: u32, //says truncated?
    pub hash: String,
    pub flags: u16,
    pub entry_path: PathBuf,
}

impl IndexEntry {
    fn read_from_index(index: &mut impl BufRead) -> Result<IndexEntry> {
        let ctime_seconds = read_u32(index).context("reading ctime seconds")?;
        let ctime_nanoseconds = read_u32(index).context("reading ctime nanoseconds")?;
        let mtime_seconds = read_u32(index).context("reading  mtime_seconds")?;
        let mtime_nanoseconds = read_u32(index).context("reading  mtime_nanoseconds")?;
        let device = read_u32(index).context("reading  device")?;
        let inode = read_u32(index).context("reading  inode")?;
        let mode = read_u32(index).context("reading  mode")?;

        let object_type = (mode >> 12) as u8;
        let unix_permission = (mode & 0x1ff) as u16;

        let user_id = read_u32(index).context("reading  user_id")?;
        let group_id = read_u32(index).context("reading  group_id")?;
        let file_size = read_u32(index).context("reading  group_id")?;

        let mut hash_buffer = [0u8; 20];
        index.read_exact(&mut hash_buffer).context("reading hash")?;
        let hash = hex::encode(&hash_buffer);
        let mut flags_buffer = [0u8; 2];
        index
            .read_exact(&mut flags_buffer)
            .context("reading flags")?;
        let flags = u16::from_be_bytes(flags_buffer);

        let mut entry_path = Vec::new();
        let entry_path_bytes = index.read_until(0, &mut entry_path)?;

        let padding = 8 - ((entry_path_bytes + 20 + 2) % 8); // "1-8 nul bytes as necessary to pad the entry to a multiple of eight bytes while keeping the name NUL-terminated."

        if padding < 8 {
            // if padding is equal to 8 that means we have no padding ie padding above
            // was 8 - 0, 0 being the amount of bytes that are not divisible by 8 so no padding
            // because divisible by 8
            for _ in 0..padding {
                index.read(&mut [0u8]).context("reading padding")?;
            }
        }
        //zeros
        entry_path.pop();

        let entry_path = OsString::from_vec(entry_path);
        let entry_path: PathBuf = entry_path.into();
        Ok(IndexEntry {
            ctime_seconds,
            ctime_nanoseconds,
            mtime_seconds,
            mtime_nanoseconds,
            device,
            inode,
            object_type,
            unix_permission,
            user_id,
            group_id,
            file_size,
            hash,
            flags,
            entry_path,
        })
    }

    pub fn from_path(path: PathBuf, hash: &str, flags: u16) -> Result<IndexEntry> {
        let metadata = fs::metadata(&path)?;
        Ok(IndexEntry {
            ctime_seconds: metadata.st_ctime() as u32,
            ctime_nanoseconds: metadata.st_ctime_nsec() as u32,
            mtime_seconds: metadata.st_mtime() as u32,
            mtime_nanoseconds: metadata.st_mtime_nsec() as u32,
            device: metadata.st_dev() as u32,
            inode: metadata.st_ino() as u32,
            object_type: match &metadata {
                //there is also something called a gitlink? not sure what
                //that is
                md if md.is_symlink() => 0b1010 as u8,
                _ => 0b1000 as u8, // for reg file
            },
            unix_permission: {
                if metadata.is_symlink() {
                    0
                } else {
                    match metadata.permissions().mode() {
                        mode if mode == 0o755 => 0o755 as u16,
                        _ => 0o644 as u16,
                    }
                }
            },
            user_id: metadata.st_uid(),
            group_id: metadata.st_gid(),
            file_size: metadata.st_size() as u32,
            hash: hash.into(),
            flags,
            entry_path: path,
        })
    }
}

impl PartialEq for IndexEntry {
    fn eq(&self, other: &Self) -> bool {
        self.mtime_seconds == other.mtime_seconds
            && self.mtime_nanoseconds == other.mtime_nanoseconds
            && self.ctime_seconds == other.ctime_seconds
            && self.ctime_nanoseconds == other.ctime_nanoseconds
            && self.device == other.device
            && self.inode == other.inode
            && self.object_type == other.object_type
            && self.unix_permission == other.unix_permission
            && self.user_id == other.user_id
            && self.group_id == other.group_id
            && self.file_size == other.file_size
            && self.entry_path == other.entry_path
    }
}

fn read_u32(bufread: &mut impl BufRead) -> Result<u32> {
    let mut buffer = [0u8; 4];
    bufread.read_exact(&mut buffer)?;
    let number = u32::from_be_bytes(buffer);
    Ok(number)
}
