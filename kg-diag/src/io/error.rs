use std::path::PathBuf;

use super::*;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum IoErrorDetail {
    Io {
        kind: std::io::ErrorKind,
    },
    IoPath {
        kind: std::io::ErrorKind,
        op_type: OpType,
        file_type: FileType,
        path: PathBuf,
    },
    CurrentDirGet {
        kind: std::io::ErrorKind,
    },
    CurrentDirSet {
        kind: std::io::ErrorKind,
        path: PathBuf,
    },
    Utf8UnexpectedEof {
        offset: usize,
    },
    Utf8InvalidEncoding {
        offset: usize,
        len: usize,
    },
    Fmt,
}

impl IoErrorDetail {
    pub fn kind(&self) -> std::io::ErrorKind {
        match *self {
            IoErrorDetail::Io { kind } => kind,
            IoErrorDetail::IoPath { kind, .. } => kind,
            IoErrorDetail::CurrentDirGet { kind, .. } => kind,
            IoErrorDetail::CurrentDirSet { kind, .. } => kind,
            IoErrorDetail::Utf8UnexpectedEof { .. } => std::io::ErrorKind::UnexpectedEof,
            IoErrorDetail::Utf8InvalidEncoding { .. } => std::io::ErrorKind::InvalidData,
            IoErrorDetail::Fmt => std::io::ErrorKind::Other,
        }
    }
    pub fn file_not_found(path: PathBuf, op_type: OpType) -> IoErrorDetail {
        IoErrorDetail::IoPath {
            kind: std::io::ErrorKind::NotFound,
            file_type: FileType::File,
            op_type,
            path,
        }
    }
}

impl Detail for IoErrorDetail {
    fn code(&self) -> u32 {
        match *self {
            IoErrorDetail::Io { kind } => 1 + kind as u32,
            IoErrorDetail::IoPath { kind, .. } => 1 + kind as u32,
            IoErrorDetail::CurrentDirGet { kind } => 1 + kind as u32,
            IoErrorDetail::CurrentDirSet { kind, .. } => 1 + kind as u32,
            IoErrorDetail::Utf8UnexpectedEof { .. } => 20,
            IoErrorDetail::Utf8InvalidEncoding { .. } => 21,
            IoErrorDetail::Fmt => 22,
        }
    }
}

impl std::fmt::Display for IoErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn kind_str(kind: std::io::ErrorKind) -> &'static str {
            use std::io::ErrorKind;
            match kind {
                ErrorKind::NotFound => "not found",
                ErrorKind::PermissionDenied => "permission denied",
                ErrorKind::ConnectionRefused => "connection refused",
                ErrorKind::ConnectionReset => "connection reset",
                ErrorKind::ConnectionAborted => "connection aborted",
                ErrorKind::NotConnected => "not connected",
                ErrorKind::AddrInUse => "address in use",
                ErrorKind::AddrNotAvailable => "address not available",
                ErrorKind::BrokenPipe => "broken pipe",
                ErrorKind::AlreadyExists => "already exists",
                ErrorKind::WouldBlock => "operation would block",
                ErrorKind::InvalidInput => "invalid input parameter",
                ErrorKind::InvalidData => "invalid data",
                ErrorKind::TimedOut => "timed out",
                ErrorKind::WriteZero => "write zero",
                ErrorKind::Interrupted => "operation interrupted",
                ErrorKind::Other => "other os error",
                ErrorKind::UnexpectedEof => "unexpected end of file",
                _ => unreachable!(),
            }
        }
        match *self {
            IoErrorDetail::Io { kind } => {
                write!(f, "{}", kind_str(kind))?;
            }
            IoErrorDetail::IoPath {
                kind,
                op_type,
                file_type,
                ref path,
            } => {
                write!(
                    f,
                    "cannot {} {} '{}': {}",
                    op_type,
                    file_type,
                    path.display(),
                    kind_str(kind)
                )?;
            }
            IoErrorDetail::CurrentDirGet { kind } => {
                write!(f, "cannot get current dir: {}", kind_str(kind))?;
            }
            IoErrorDetail::CurrentDirSet { kind, ref path } => {
                write!(
                    f,
                    "cannot set current dir to {}: {}",
                    path.display(),
                    kind_str(kind)
                )?;
            }
            IoErrorDetail::Utf8UnexpectedEof { offset, .. } => {
                write!(
                    f,
                    "unexpected end of input at offset {} while decoding utf-8",
                    offset
                )?;
            }
            IoErrorDetail::Utf8InvalidEncoding { offset, .. } => {
                write!(f, "invalid utf-8 encoding at offset {}", offset)?;
            }
            IoErrorDetail::Fmt => {
                write!(f, "formatting error")?;
            }
        }
        Ok(())
    }
}

impl From<std::io::Error> for IoErrorDetail {
    fn from(err: std::io::Error) -> Self {
        IoErrorDetail::Io { kind: err.kind() }
    }
}

impl From<std::io::ErrorKind> for IoErrorDetail {
    fn from(kind: std::io::ErrorKind) -> Self {
        IoErrorDetail::Io { kind }
    }
}

impl From<std::fmt::Error> for IoErrorDetail {
    fn from(_: std::fmt::Error) -> Self {
        IoErrorDetail::Fmt
    }
}

pub trait ResultExt<T> {
    fn info<P: Into<PathBuf>>(self, path: P, op_type: OpType, file_type: FileType) -> IoResult<T>;
}

impl<T> ResultExt<T> for std::io::Result<T> {
    #[inline]
    fn info<P: Into<PathBuf>>(self, path: P, op_type: OpType, file_type: FileType) -> IoResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(IoErrorDetail::IoPath {
                kind: err.kind(),
                op_type,
                file_type,
                path: path.into(),
            }),
        }
    }
}
