use super::*;

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


#[derive(Display, Debug, Eq, PartialEq, Clone, Copy)]
pub enum NumericalErrorKind {
    #[display("overflow")]
    Overflow,
    #[display("underflow")]
    Underflow,
    #[display("invalid format error")]
    Invalid,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ParseErrorDetail {
    Io(IoErrorDetail),
    UnexpectedEof {
        pos: Position,
        expected: Option<Expected>,
        task: String,
    },
    UnexpectedInput {
        pos: Position,
        found: Option<Input>,
        expected: Option<Expected>,
        task: String,
    },
    Numerical {
        span: Span,
        kind: NumericalErrorKind,
    }
}

impl Detail for ParseErrorDetail {
    fn code(&self) -> u32 {
        match *self {
            ParseErrorDetail::Io(ref err) => err.code(),
            ParseErrorDetail::UnexpectedEof { .. } => 40,
            ParseErrorDetail::UnexpectedInput { .. } => 41,
            ParseErrorDetail::Numerical { .. } => 42,
        }
    }
}

impl std::fmt::Display for ParseErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ParseErrorDetail::Io(ref err) => {
                return std::fmt::Display::fmt(err, f);
            }
            ParseErrorDetail::UnexpectedEof { pos, ref expected, ref task } => {
                write!(f, "unexpected <EOF> at {} while {}", pos, task)?;
                if let Some(e) = expected {
                    write!(f, ", expecting {}", e)?;
                }
            }
            ParseErrorDetail::UnexpectedInput { pos, ref found, ref expected, ref task } => {
                if let Some(ref input) = found {
                    write!(f, "unexpected {} at {} while {}", input, pos, task)?;
                } else {
                    write!(f, "unexpected input at {} while {}", pos, task)?;
                }
                if let Some(e) = expected {
                    write!(f, ", expecting {}", e)?;
                }
            }
            ParseErrorDetail::Numerical { span, kind } => {
                write!(f, "{} while converting number literal at {}", kind, span)?;
            }
        }
        Ok(())
    }
}

impl From<IoErrorDetail> for ParseErrorDetail {
    fn from(err: IoErrorDetail) -> Self {
        ParseErrorDetail::Io(err)
    }
}
