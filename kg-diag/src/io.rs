use std;
use std::path::{Path, PathBuf};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Position {
    pub offset: usize,
    pub line: u32,
    pub column: u32,
}

#[allow(unused)]
impl Position {
    pub fn new() -> Position {
        Position {
            offset: 0,
            line: 0,
            column: 0,
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

#[derive(Debug, Clone)]
pub struct Quote {
    path: Option<PathBuf>,
    from: Position,
    to: Position,
    offset: usize,
    line: u32,
    source: String,
    message: String,
}

#[allow(unused)]
impl Quote {
    pub fn new<'a>(path: Option<&Path>, data: &[u8], from: Position, to: Position,
                   lines_before: u32, lines_after: u32, message: Cow<'a, str>) -> Quote {
        let mut line = 0;
        let mut off1 = 0;
        let mut off2 = data.len();
        let mut lines = 0;

        let before = &data[0..from.offset];
        for (p, c) in before.iter().rev().enumerate() {
            if *c == b'\n' {
                if lines < lines_before {
                    lines += 1;
                } else {
                    off1 = from.offset - p;
                    line = from.line - lines_before;
                    break;
                }
            }
        }

        let after = &data[to.offset..];
        lines = 0;
        for (p, c) in after.iter().enumerate() {
            if *c == b'\n' {
                if lines < lines_after {
                    lines += 1;
                } else {
                    off2 = to.offset + p;
                    break;
                }
            }
        }

        Quote {
            path: path.map(|p| p.to_path_buf()),
            from,
            to,
            offset: off1,
            line,
            source: String::from_utf8_lossy(&data[off1..off2]).into(),
            message: message.into(),
        }
    }

    pub fn from(&self) -> Position {
        self.from
    }

    pub fn to(&self) -> Position {
        self.to
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
            cmp::max(((self.line + self.source.len() as u32 + 1) as f64).log10().ceil() as usize, 3)
        } else {
            0
        };
        let mut ln = self.line;
        if self.path.is_some() {
            write!(f, "{0:>1$} {2}:{3}\n", " -->", line_chars, self.path.as_ref().unwrap().to_str().unwrap(), self.from)?;
        }
        for s in self.source.lines() {
            if show_line_numbers {
                write!(f, "{0:>1$}| ", ln + 1, line_chars)?;
            }
            if ln >= self.from.line && ln <= self.to.line {
                write!(f, "{}\n", s)?;
                if show_line_numbers {
                    write!(f, "{0:1$}| ", " ", line_chars)?;
                }
                for _ in 0..self.from.column {
                    write!(f, " ")?;
                }
                for _ in self.from.column..self.to.column {
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
pub trait LexTerm: std::fmt::Debug + std::fmt::Display + PartialEq + Eq + Sync + Send + 'static {}


/// Generic token structure (i.e. terminal along with it's location in source)
#[derive(Debug, Display, Clone, Copy)]
#[display(fmt = "{term}")]
pub struct LexToken<T: LexTerm + Clone + Copy> {
    term: T,
    from: Position,
    to: Position,
}

impl<T: LexTerm + Clone + Copy> LexToken<T> {
    pub fn new(term: T, from: Position, to: Position) -> LexToken<T> {
        LexToken {
            term,
            from,
            to,
        }
    }

    pub fn term(&self) -> T {
        self.term
    }

    pub fn from(&self) -> Position {
        self.from
    }

    pub fn to(&self) -> Position {
        self.to
    }
}
