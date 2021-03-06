use std::fs::ReadDir;
use std::fs::{File, Metadata, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use super::*;

#[derive(Debug, Display, Clone, Copy, Eq, PartialEq, Hash)]
pub enum OpType {
    #[display(fmt = "create")]
    Create,
    #[display(fmt = "read")]
    Read,
    #[display(fmt = "write")]
    Write,
    #[display(fmt = "remove")]
    Remove,
    #[display(fmt = "stat")]
    Stat,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    #[display(fmt = "path")]
    Unknown,
    #[display(fmt = "file")]
    File,
    #[display(fmt = "dir")]
    Dir,
    #[display(fmt = "link")]
    Link,
    #[display(fmt = "device")]
    Device,
    #[display(fmt = "special file")]
    Special,
}

impl FileType {
    pub fn is_file(&self) -> bool {
        self == &FileType::File
    }
}

impl From<std::fs::FileType> for FileType {
    fn from(f: std::fs::FileType) -> FileType {
        if f.is_dir() {
            FileType::Dir
        } else if f.is_file() {
            FileType::File
        } else if f.is_symlink() {
            FileType::Link
        } else {
            unreachable!();
        }
    }
}

#[derive(Debug)]
pub struct FileBuffer {
    data: Vec<u8>,
    path: PathBuf,
}

impl FileBuffer {
    pub fn open<P: Into<PathBuf> + AsRef<Path>>(path: P) -> IoResult<FileBuffer> {
        let mut f = File::open(path.as_ref()).info(path.as_ref(), OpType::Read, FileType::File)?;
        let m = f
            .metadata()
            .info(path.as_ref(), OpType::Read, FileType::File)?;
        let mut data: Vec<u8> = Vec::with_capacity(m.len() as usize);
        f.read_to_end(&mut data)
            .info(path.as_ref(), OpType::Read, FileType::File)?;
        Ok(FileBuffer {
            data,
            path: path.into(),
        })
    }

    pub fn create<P: Into<PathBuf> + AsRef<Path>>(path: P) -> IoResult<FileBuffer> {
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path.as_ref())
            .info(path.as_ref(), OpType::Create, FileType::File)?;
        Ok(FileBuffer {
            data: Vec::new(),
            path: path.into(),
        })
    }

    pub fn char_reader(&self) -> MemCharReader {
        MemCharReader::with_path(&self.path, &self.data)
    }

    pub fn byte_reader(&self) -> MemByteReader {
        MemByteReader::with_path(&self.path, &self.data)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn write(&mut self, data: &[u8]) -> IoResult<()> {
        self.data = data.to_owned();
        let mut f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .info(&self.path, OpType::Write, FileType::File)?;
        f.write(&self.data)
            .info(&self.path, OpType::Write, FileType::File)?;
        f.sync_data()
            .info(&self.path, OpType::Write, FileType::File)?;
        Ok(())
    }

    pub fn into_data(self) -> Vec<u8> {
        self.data
    }
}

pub fn read_to_string<P: AsRef<Path>>(file_path: P, buf: &mut String) -> IoResult<()> {
    let mut f =
        File::open(file_path.as_ref()).info(file_path.as_ref(), OpType::Read, FileType::File)?;
    buf.reserve_exact(
        f.metadata()
            .info(file_path.as_ref(), OpType::Read, FileType::File)?
            .len() as usize,
    );
    f.read_to_string(buf)
        .info(file_path.as_ref(), OpType::Read, FileType::File)?;
    Ok(())
}

pub fn read_string<P: AsRef<Path>>(file_path: P) -> IoResult<String> {
    let mut s = String::new();
    read_to_string(file_path, &mut s)?;
    Ok(s)
}

pub fn remove_file<P: Into<PathBuf> + AsRef<Path>>(file_path: P) -> IoResult<()> {
    std::fs::remove_file(file_path.as_ref()).info(file_path, OpType::Remove, FileType::File)?;
    Ok(())
}

pub fn canonicalize<P: AsRef<Path>>(file_path: P) -> IoResult<PathBuf> {
    Ok(std::fs::canonicalize(file_path.as_ref()).info(
        file_path.as_ref(),
        OpType::Read,
        FileType::Unknown,
    )?)
}

pub fn current_dir() -> IoResult<PathBuf> {
    match std::env::current_dir() {
        Ok(dir) => Ok(dir),
        Err(err) => {
            let e = IoErrorDetail::CurrentDirGet { kind: err.kind() };
            Err(e)
        }
    }
}

pub fn read_dir<P: AsRef<Path>>(path: P) -> IoResult<ReadDir> {
    std::fs::read_dir(path.as_ref()).info(path.as_ref(), OpType::Read, FileType::Dir)
}

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> IoResult<()> {
    std::fs::write(path.as_ref(), contents).info(path.as_ref(), OpType::Write, FileType::File)
}

pub fn create_dir<P: Into<PathBuf> + AsRef<Path>>(dir: P) -> IoResult<()> {
    std::fs::create_dir(dir.as_ref()).info(dir, OpType::Create, FileType::Dir)?;
    Ok(())
}

pub fn create_dir_all<P: Into<PathBuf> + AsRef<Path>>(dir: P) -> IoResult<()> {
    let mut paths: Vec<_> = dir.as_ref().ancestors().collect();
    paths.pop();
    while let Some(p) = paths.pop() {
        if !p.exists() {
            create_dir(p)?;
        } else if !p.is_dir() {
            return Err(IoErrorDetail::IoPath {
                kind: std::io::ErrorKind::AlreadyExists,
                path: p.into(),
                op_type: OpType::Create,
                file_type: FileType::Dir,
            }
            .into());
        }
    }
    Ok(())
}

pub fn remove_dir<P: Into<PathBuf> + AsRef<Path>>(dir: P) -> IoResult<()> {
    std::fs::remove_dir(dir.as_ref()).info(dir, OpType::Remove, FileType::Unknown)?;
    Ok(())
}

pub fn remove_dir_all<P: Into<PathBuf> + AsRef<Path>>(dir: P) -> IoResult<()> {
    clear_dir_all(dir.as_ref())?;
    remove_dir(dir)
}

pub fn clear_dir_all<P: Into<PathBuf> + AsRef<Path>>(dir: P) -> IoResult<()> {
    let d = read_dir(dir.as_ref())?;
    for e in d {
        let e = e.info(dir.as_ref(), OpType::Read, FileType::Dir)?;
        let ft = e.file_type().info(e.path(), OpType::Stat, FileType::Unknown)?;
        if ft.is_dir() {
            remove_dir_all(e.path())?;
        } else if ft.is_symlink() {
            let path = e.path();
            std::fs::remove_file(&path).info(path, OpType::Remove, FileType::Link)?;
        } else {
            remove_file(e.path())?;
        }
    }
    Ok(())
}

pub fn metadata<P: AsRef<Path>>(path: P) -> IoResult<Metadata> {
    std::fs::metadata(path.as_ref()).info(path.as_ref(), OpType::Read, FileType::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_not_found() {
        let e = FileBuffer::open("./should_not_exist").unwrap_err();
        let err = error::IoErrorDetail::IoPath {
            path: std::path::PathBuf::from("./should_not_exist"),
            op_type: OpType::Read,
            file_type: FileType::File,
            kind: std::io::ErrorKind::NotFound,
        };

        assert_eq!(e, err);
    }

    #[test]
    fn canonicalize() {
        let err = fs::canonicalize("./should_not_exist").unwrap_err();
        assert_eq!(
            err,
            error::IoErrorDetail::IoPath {
                kind: std::io::ErrorKind::NotFound,
                op_type: OpType::Read,
                file_type: FileType::Unknown,
                path: std::path::PathBuf::from("./should_not_exist")
            }
        );
    }

    #[test]
    fn read_to_string() {
        use std::io::{Seek, SeekFrom};
        use tempfile::NamedTempFile;

        let mut tmpfile = NamedTempFile::new().unwrap();
        tmpfile.write_all(b"\xe2\x28\xa1").unwrap();
        tmpfile.seek(SeekFrom::Start(0)).unwrap();

        let mut s = String::new();
        let path = tmpfile.path();
        let err = fs::read_to_string(path, &mut s).unwrap_err();
        assert_eq!(
            err,
            error::IoErrorDetail::IoPath {
                kind: std::io::ErrorKind::InvalidData,
                op_type: OpType::Read,
                file_type: FileType::File,
                path: std::path::PathBuf::from(path)
            }
        );
    }

    #[test]
    fn current_dir() {
        let path = std::env::current_dir().unwrap();
        let s = path.join("tempDir");
        std::fs::create_dir(&s).unwrap();
        std::env::set_current_dir(&s).unwrap();
        std::fs::remove_dir(s).unwrap();
        let err = fs::current_dir().unwrap_err();
        assert_eq!(
            err,
            error::IoErrorDetail::CurrentDirGet {
                kind: std::io::ErrorKind::NotFound
            }
        );
        std::env::set_current_dir(&path).unwrap();
    }
}
