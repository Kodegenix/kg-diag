#![feature(specialization)]

extern crate kg_diag;

use kg_diag::*;

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
        write!(
            f,
            "invalid token '{}', expected one of {:?}",
            self.found, self.expected
        )
    }
}

fn error_fun() -> Result<(), SimpleDiag> {
    Err(InvalidToken {
        expected: &["id", "num", "+", "-"],
        found: "*".into(),
    }
    .into())
}

fn main() {
    let err = error_fun().unwrap_err();
    println!("{}", err);
}
