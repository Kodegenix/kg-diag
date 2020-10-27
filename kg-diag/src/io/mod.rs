pub use self::error::{IoErrorDetail, ResultExt};
pub use self::fs::{FileBuffer, FileType, OpType};
pub use self::reader::{ByteReader, CharReader, MemByteReader, MemCharReader, Reader};

pub mod error;
pub mod fs;
mod reader;

pub type IoResult<T> = std::result::Result<T, IoErrorDetail>;

use super::*;

use std;
use std::borrow::Cow;
use std::path::{Path, PathBuf};


#[repr(C)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Position {
    pub offset: usize,
    pub line: u32,
    pub column: u32,
}

impl Position {
    pub fn new() -> Position {
        Position {
            offset: 0,
            line: 0,
            column: 0,
        }
    }
    pub fn with(offset: usize, line: u32, column: u32) -> Position {
        Position {
            offset,
            line,
            column,
        }
    }

    #[inline]
    pub fn inc_column(&mut self) {
        self.column += 1;
    }

    #[inline]
    pub fn inc_line(&mut self) {
        self.line += 1;
        self.column = 0;
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.column + 1)
    }
}

impl Default for Position {
    fn default() -> Position {
        Position::new()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new() -> Span {
        Span {
            start: Position::new(),
            end: Position::new(),
        }
    }

    pub fn with(
        start_offset: usize,
        start_line: u32,
        start_column: u32,
        end_offset: usize,
        end_line: u32,
        end_column: u32,
    ) -> Span {
        let start = Position::with(start_offset, start_line, start_column);
        let end = Position::with(end_offset, end_line, end_column);
        Self::with_pos(start, end)
    }

    pub fn with_pos(start: Position, end: Position) -> Span {
        Span {
            start,
            end,
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if f.alternate() || self.start.line != self.end.line {
            write!(f, "{}-{}", self.start, self.end)
        } else {
            write!(
                f,
                "{}:{}-{}",
                self.start.line + 1,
                self.start.column + 1,
                self.end.column + 1
            )
        }
    }
}

impl Default for Span {
    fn default() -> Span {
        Span::new()
    }
}

#[derive(Debug, Clone)]
pub struct Quote {
    path: Option<PathBuf>,
    span: Span,
    offset: usize,
    line: u32,
    source: String,
    message: String,
}

#[allow(unused)]
impl Quote {
    pub fn new<'a>(
        path: Option<&Path>,
        data: &[u8],
        start: Position,
        end: Position,
        lines_before: u32,
        lines_after: u32,
        message: Cow<'a, str>,
    ) -> Quote {
        let mut line = 0;
        let mut off1 = 0;
        let mut off2 = data.len();
        let mut lines = 0;

        let before = &data[0..start.offset];
        for (p, c) in before.iter().rev().enumerate() {
            if *c == b'\n' {
                if lines < lines_before {
                    lines += 1;
                } else {
                    off1 = start.offset - p;
                    line = start.line - lines_before;
                    break;
                }
            }
        }

        let after = &data[end.offset..];
        lines = 0;
        for (p, c) in after.iter().enumerate() {
            if *c == b'\n' {
                if lines < lines_after {
                    lines += 1;
                } else {
                    off2 = end.offset + p;
                    break;
                }
            }
        }

        Quote {
            path: path.map(|p| p.to_path_buf()),
            span: Span::with_pos(start, end),
            offset: off1,
            line,
            source: String::from_utf8_lossy(&data[off1..off2]).into(),
            message: message.into(),
        }
    }

    pub fn start(&self) -> Position {
        self.span.start
    }

    pub fn end(&self) -> Position {
        self.span.end
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

impl std::fmt::Display for Quote {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::cmp;

        let show_line_numbers = self.path.is_some() || self.line != 0 || self.source.len() > 1;
        let line_chars = if show_line_numbers {
            cmp::max(
                ((self.line + self.source.len() as u32 + 1) as f64)
                    .log10()
                    .ceil() as usize,
                3,
            )
        } else {
            0
        };
        let mut ln = self.line;
        if self.path.is_some() {
            write!(
                f,
                "{0:>1$} {2}:{3}\n",
                " -->",
                line_chars,
                self.path.as_ref().unwrap().to_str().unwrap(),
                self.span.start
            )?;
        }
        for s in self.source.lines() {
            if show_line_numbers {
                write!(f, "{0:>1$}| ", ln + 1, line_chars)?;
            }
            if ln == self.span.start.line && ln == self.span.end.line {
                write!(f, "{}\n", s)?;
                if show_line_numbers {
                    write!(f, "{0:1$}| ", " ", line_chars)?;
                }
                for _ in 0..self.span.start.column {
                    write!(f, " ")?;
                }
                for _ in self.span.start.column..self.span.end.column {
                    write!(f, "^")?;
                }
                write!(f, " {}\n", self.message)?;
            } else {
                write!(f, "{}\n", s)?;
            }
            ln += 1;
        }
        Ok(())
    }
}

/// Marker trait representing terminals used in parsing
pub trait LexTerm:
    std::fmt::Debug + std::fmt::Display + PartialEq + Eq + Sync + Send + 'static
{
}

/// Generic token structure (i.e. terminal along with it's location in source)
#[derive(Debug, Display, Clone, Copy)]
#[display(fmt = "{term}")]
pub struct LexToken<T: LexTerm + Clone + Copy> {
    term: T,
    span: Span,
}

impl<T: LexTerm + Clone + Copy> LexToken<T> {
    pub fn new(term: T, from: Position, to: Position) -> LexToken<T> {
        LexToken {
            term,
            span: Span { start: from, end: to },
        }
    }

    pub fn term(&self) -> T {
        self.term
    }

    pub fn start(&self) -> Position {
        self.span.start
    }

    pub fn end(&self) -> Position {
        self.span.end
    }

    pub fn span(&self) -> Span {
        self.span
    }
}
