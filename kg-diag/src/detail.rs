use std::any::TypeId;
use std::convert::TryFrom;
use std::fmt::{Debug, Display};
use crate::{BasicDiag, Diag};

#[derive(Debug, Display, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Severity {
    #[display("info")]
    Info,

    #[display("warning")]
    Warning,

    /// error that is recoverable (i.e. process might continue and check additional diagnostics)
    #[display("error")]
    Error,

    /// non-recoverable error
    #[display("error")]
    Failure,

    /// fatal error, usually terminating the process abnormally (like OOM errors)
    #[display("critical error")]
    Critical,
}

impl Severity {
    pub fn code_byte(&self) -> u8 {
        match *self {
            Severity::Info => b'I',
            Severity::Warning => b'W',
            Severity::Error => b'E',
            Severity::Failure => b'F',
            Severity::Critical => b'C',
        }
    }

    pub fn code_char(&self) -> char {
        self.code_byte() as char
    }

    pub fn is_error(&self) -> bool {
        *self >= Severity::Error
    }

    pub fn is_recoverable(&self) -> bool {
        *self < Severity::Failure
    }
}

impl<'a> TryFrom<&'a str> for Severity {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<Self, <Self as TryFrom<&'a str>>::Error> {
        if value.eq_ignore_ascii_case("info") {
            Ok(Severity::Info)
        } else if value.eq_ignore_ascii_case("warning") {
            Ok(Severity::Warning)
        } else if value.eq_ignore_ascii_case("error") {
            Ok(Severity::Error)
        } else if value.eq_ignore_ascii_case("failure") {
            Ok(Severity::Failure)
        } else if value.eq_ignore_ascii_case("critical") {
            Ok(Severity::Critical)
        } else {
            Err(value)
        }
    }
}

impl TryFrom<char> for Severity {
    type Error = char;

    fn try_from(value: char) -> Result<Self, <Self as TryFrom<char>>::Error> {
        Ok(match value.to_ascii_uppercase() {
            'I' => Severity::Info,
            'W' => Severity::Warning,
            'E' => Severity::Error,
            'F' => Severity::Failure,
            'C' => Severity::Critical,
            _ => return Err(value),
        })
    }
}

pub trait Detail: Display + Debug + Send + Sync + 'static {
    fn severity(&self) -> Severity;

    fn code(&self) -> u32;

    fn type_id(&self) -> TypeId;

    fn as_fmt_debug(&self) -> &dyn std::fmt::Debug;

    fn as_fmt_display(&self) -> &dyn std::fmt::Display;
}

impl<T: Detail> Detail for T {
    default fn severity(&self) -> Severity {
        Severity::Failure
    }

    default fn code(&self) -> u32 {
        0
    }

    default fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn as_fmt_debug(&self) -> &dyn std::fmt::Debug {
        self as &dyn std::fmt::Debug
    }

    fn as_fmt_display(&self) -> &dyn std::fmt::Display {
        self as &dyn std::fmt::Display
    }
}

impl dyn Detail {
    pub fn downcast_ref<T: Detail>(&self) -> Option<&T> {
        if self.type_id() == TypeId::of::<T>() {
            unsafe { Some(&*(self as *const dyn Detail as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: Detail>(&mut self) -> Option<&mut T> {
        if self.type_id() == TypeId::of::<T>() {
            unsafe { Some(&mut *(self as *mut dyn Detail as *mut T)) }
        } else {
            None
        }
    }
}

pub trait DetailExt {
    fn with_cause<D: Diag>(self, cause: D) -> BasicDiag;
}

impl <T> DetailExt for T where T: Detail {
    fn with_cause<D: Diag>(self, cause: D) -> BasicDiag {
        BasicDiag::with_cause(self, cause)
    }
}

impl Detail for String { }
