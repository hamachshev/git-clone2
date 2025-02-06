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
        index.read_until(0, &mut entry_path)?;
        let mut peekreader = PeekReader::new(index);

        while peekreader.peek_and_delete_0().context("peeking for 0s")? == 0x00 {} //delete all the padding
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

struct PeekReader<R>
where
    R: BufRead,
{
    peek_buffer: Vec<u8>,
    pointer: usize,
    reader: R,
}

impl<R> PeekReader<R>
where
    R: BufRead,
{
    fn new(reader: R) -> PeekReader<R> {
        PeekReader {
            reader,
            pointer: 0usize,
            peek_buffer: Vec::new(),
        }
    }

    fn peek_and_delete_0(&mut self) -> Result<u8> {
        let mut buffer = [0u8];
        self.reader.read(&mut buffer)?;
        if buffer[0] != 0 {
            self.peek_buffer.push(buffer[0].clone());
        }
        Ok(buffer[0])
    }
}

impl<R> Read for PeekReader<R>
where
    R: BufRead,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.peek_buffer.is_empty() {
            return self.reader.read(buf);
        } else {
            match self.peek_buffer.len() {
                len if (len - self.pointer) < buf.len() => {
                    buf[..len].copy_from_slice(&self.peek_buffer[self.pointer..]);
                    self.peek_buffer.clear();
                    self.pointer = 0;
                    self.reader.read(&mut buf[len..])
                }
                len if (len - self.pointer) > buf.len() => {
                    buf.copy_from_slice(
                        &self.peek_buffer[self.pointer..(self.pointer + buf.len())],
                    );
                    self.pointer += buf.len();
                    Ok(buf.len())
                }
                len if (len - self.pointer) == buf.len() => {
                    buf.copy_from_slice(&self.peek_buffer[self.pointer..]);
                    Ok(buf.len())
                }
                _ => Err(Error::new(
                    ErrorKind::Other,
                    "this will not happen".to_string(),
                )),
            }
        }
    }
}
