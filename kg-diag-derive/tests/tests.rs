#![feature(specialization, attr_literals)]

extern crate kg_diag;
#[macro_use]
extern crate kg_diag_derive;
#[macro_use]
extern crate kg_display_derive;

use kg_diag::*;

#[allow(unused)]
#[derive(Debug, Detail, Display)]
#[diag(code_offset = 1000)]
enum TestErrorKind {
    #[diag(code = 1, severity = 'E')]
    #[display(fmt = "empty error message")]
    ErrorEmpty,

    #[diag(code = 2, severity = 'F')]
    #[display(fmt = "error with pair of {a0} and {a1}")]
    ErrorWithPair(usize, usize),

    #[diag(severity = "error")]
    #[display(fmt = "error with \"{a0}\" string")]
    ErrorWithString(String),

    #[diag(code = 4, severity = "failure")]
    #[display(fmt = "error with field a = {a} and field b = {b}")]
    ErrorWithStruct {
        a: usize,
        b: usize,
    }
}


#[test]
fn code_deref() {
    let e = TestErrorKind::ErrorWithString("string value".into());
    println!("{}", e.code());
    println!("{}", e.severity());
    println!("{}", e);
}

