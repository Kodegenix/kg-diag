#![feature(box_syntax, specialization, raw)]

#[macro_use]
extern crate kg_display_derive;

pub use self::detail::{Detail, Severity};
pub use self::diag::{BasicDiag, Diag, ParseDiag, SimpleDiag};
pub use self::io::{LexTerm, LexToken, Position, Span, Quote};
pub use self::multi::{Diags, Errors};
pub use self::stacktrace::Stacktrace;

mod io;
mod detail;
mod diag;
mod multi;
mod stacktrace;

#[macro_export]
macro_rules! basic_diag {
    ($kind: expr) => {{
        $crate::BasicDiag::from($kind)
    }};
    ($logger: expr, $kind: expr) => {{
        let e = $crate::BasicDiag::from($kind);
        slog_debug!($logger, "diagnostic created:\n{}", e);
        e
    }};
}

#[macro_export]
macro_rules! parse_diag {
    ($kind: expr) => {{
        $crate::ParseDiag::from($kind)
    }};
    ($kind: expr, $reader: expr, { $($p1: expr, $p2: expr => $msg: expr),+ $(,)* }) => {{
        let mut e = $crate::ParseDiag::from($kind);
        $(
        e.add_quote($reader.quote($p1, $p2, 2, 2, $msg.into()));
        )+
        e
    }};
    ($logger: expr, $kind: expr) => {{
        let e = $crate::ParseDiag::from($kind);
        slog_debug!("parse diagnostic created:\n{}", e);
        e
    }};
    ($logger: expr, $kind: expr, $reader: expr, { $($p1: expr, $p2: expr => $msg: expr),+ $(,)* }) => {{
        let mut e = $crate::ParseDiag::from($kind);
        $(
        e.add_quote($reader.quote($p1, $p2, 2, 2, $msg.into()));
        )+
        slog_debug!("parse diagnostic created:\n{}", e);
        e
    }};
}

pub trait ResultExt<T> {
    fn into_diag(self) -> Result<T, BasicDiag>;
}

impl<T, E: Detail> ResultExt<T> for Result<T, E> {
    fn into_diag(self) -> Result<T, BasicDiag> {
        self.map_err(|detail| BasicDiag::from(detail))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detail_debug() {
        #[derive(Debug)]
        struct InvalidToken {
            expected: &'static [&'static str],
            found: String,
        }

        impl Detail for InvalidToken {
            fn severity(&self) -> Severity {
                Severity::Error
            }

            fn code(&self) -> u32 {
                101
            }
        }

        impl std::fmt::Display for InvalidToken {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "invalid token '{}', expected one of {:?}", self.found, self.expected)
            }
        }

        let err: ParseDiag = InvalidToken {
            expected: &["id", "num", "+", "-"],
            found: "*".into(),
        }.into();

        println!("{:#?}", err);
        println!("{}", err);
    }
}

