use std::io::{Error, ErrorKind};
use std::os::unix::ffi::OsStringExt;
use std::{
    ffi::OsString,
    fs,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
};

use anyhow::{Context, Result};

pub fn invoke(file: &std::path::PathBuf) -> Result<()> {
    if fs::exists(".git/index").is_err() {
        fs::File::create_new(".git/index");
    }

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
        let entry = IndexFile::read_from_index(&mut bufread)?;
        entries.push(entry);
    }

    println!("{:#?}", entries);

    Ok(())
}

#[derive(Debug)]
struct IndexFile {
    //https://git-scm.com/docs/index-format
    ctime_seconds: u32,
    ctime_nanoseconds: u32,
    mtime_seconds: u32,
    mtime_nanoseconds: u32,
    device: u32,
    inode: u32,
    object_type: u8,      // 4bit valid types are in binary 1000, 1010, 1110
    unix_permission: u16, // 9 bit valid types 0775 and 0644 in octal
    // permission
    user_id: u32,
    group_id: u32,
    file_size: u32, //says truncated?
    hash: String,
    flags: u16,
    entry_path: PathBuf,
}

impl IndexFile {
    fn read_from_index(index: &mut impl BufRead) -> Result<IndexFile> {
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
        Ok(IndexFile {
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
}

fn read_u32(bufread: &mut impl BufRead) -> Result<u32> {
    let mut buffer = [0u8; 4];
    bufread.read_exact(&mut buffer)?;
    let number = u32::from_be_bytes(buffer);
    Ok(number)
}
