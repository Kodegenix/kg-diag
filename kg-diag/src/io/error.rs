use super::*;

use std::path::PathBuf;
use kg_display::ListDisplay;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub enum Input {
    Byte(u8),
    Char(char),
    Custom(String),
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Input::Byte(b) => write!(f, "byte 0x{:02X}", b),
            Input::Char(c) => write!(f, "character {:?}", c),
            Input::Custom(ref s) => write!(f, "{}", s),
        }
    }
}


#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub enum Expected {
    Byte(u8),
    ByteRange(u8, u8),
    Char(char),
    CharRange(char, char),
    Custom(String),
    OneOf(Vec<Expected>),
    Or(Box<Expected>, Box<Expected>),
}

impl Expected {
    pub fn one_of(mut elems: Vec<Expected>) -> Expected {
        if elems.len() == 1 {
            elems.pop().unwrap()
        } else {
            elems.sort();
            Expected::OneOf(elems)
        }
    }
}

impl std::fmt::Display for Expected {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Expected::Byte(b) => write!(f, "0x{:02X}", b),
            Expected::ByteRange(a, b) => write!(f, "[0x{:02X}-0x{:02X}]", a, b),
            Expected::Char(c) => write!(f, "{:?}", c),
            Expected::CharRange(a, b) => write!(f, "[{:?}-{:?}]", a, b),
            Expected::Custom(ref s) => write!(f, "{}", s),
            Expected::OneOf(ref e) => write!(f, "one of: {}", ListDisplay(e)),
            Expected::Or(ref a, ref b) => write!(f, "{} or {}", a, b),
        }
    }
}


#[derive(Debug, Eq, PartialEq, Clone)]
pub enum IoErrorDetail {
    Io {
        kind: std::io::ErrorKind,
        message: String,
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
    Utf8InvalidEncoding {
        pos: Position,
        len: usize,
    },
    UnexpectedEof {
        pos: Position,
        expected: Option<Box<Expected>>,
        task: String,
    },
    UnexpectedInput {
        pos: Position,
        found: Input,
        expected: Option<Box<Expected>>,
        task: String,
    },
    Fmt,
}

impl IoErrorDetail {
    pub fn kind(&self) -> std::io::ErrorKind {
        match *self {
            IoErrorDetail::Io { kind, .. } => kind,
            IoErrorDetail::IoPath { kind, .. } => kind,
            IoErrorDetail::CurrentDirGet { kind, .. } => kind,
            IoErrorDetail::CurrentDirSet { kind, .. } => kind,
            IoErrorDetail::Utf8InvalidEncoding { .. } => std::io::ErrorKind::InvalidData,
            IoErrorDetail::UnexpectedEof { .. } => std::io::ErrorKind::UnexpectedEof,
            IoErrorDetail::UnexpectedInput { .. } => std::io::ErrorKind::InvalidData,
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
            IoErrorDetail::Io { kind, message: _ } => 1 + kind as u32,
            IoErrorDetail::IoPath { kind, .. } => 1 + kind as u32,
            IoErrorDetail::CurrentDirGet { kind } => 1 + kind as u32,
            IoErrorDetail::CurrentDirSet { kind, .. } => 1 + kind as u32,
            IoErrorDetail::Utf8InvalidEncoding { .. } => 21,
            IoErrorDetail::UnexpectedEof { .. } => 22,
            IoErrorDetail::UnexpectedInput { .. } => 23,
            IoErrorDetail::Fmt => 99,
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
            IoErrorDetail::Io { kind, ref message } => {
                write!(f, "{}", kind_str(kind))?;
                if !message.is_empty() {
                    write!(f, ": {}", message)?;
                }
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
            IoErrorDetail::Utf8InvalidEncoding { pos, .. } => {
                write!(f, "invalid utf-8 encoding at {} (offset {})", pos, pos.offset)?;
            }
            IoErrorDetail::UnexpectedEof { pos, ref expected, ref task } => {
                write!(f, "unexpected end of input at {} while {}", pos, task)?;
                if let Some(e) = expected {
                    write!(f, ", expecting {}", e)?;
                }
            }
            IoErrorDetail::UnexpectedInput { pos, ref found, ref expected, ref task } => {
                write!(f, "unexpected {} at {} while {}", found, pos, task)?;
                if let Some(e) = expected {
                    write!(f, ", expecting {}", e)?;
                }
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
        if let Some(e) = err.get_ref() {
            IoErrorDetail::Io {
                kind: err.kind(),
                message: format!("{}", e)
            }
        } else {
            IoErrorDetail::Io {
                kind: err.kind(),
                message: String::new()
            }
        }

    }
}

impl From<std::io::ErrorKind> for IoErrorDetail {
    fn from(kind: std::io::ErrorKind) -> Self {
        IoErrorDetail::Io {
            kind,
            message: String::new()
        }
    }
}

impl From<std::fmt::Error> for IoErrorDetail {
    fn from(_: std::fmt::Error) -> Self {
        IoErrorDetail::Fmt
    }
}

pub trait ResultExt<T> {
    /// Add additional information to underlining `std::io::Error` and map this error to `IoErrorDetail`
    fn info<P: Into<PathBuf>>(self, path: P, op_type: OpType, file_type: FileType) -> IoResult<T>;

    /// Convert `std::io::Error` into `BasicDiag`
    fn map_err_to_diag(self) -> Result<T, BasicDiag>;
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

    fn map_err_to_diag(self) -> Result<T, BasicDiag> {
        self.map_err(|err| IoErrorDetail::from(err))
            .into_diag_res()
    }
}
