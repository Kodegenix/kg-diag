#![feature(specialization)]

#[macro_use]
extern crate kg_diag;
extern crate kg_io;

use kg_diag::{Detail, Diag};
use kg_io::*;

#[derive(Debug)]
enum TestErrorKind {
    ErrorEmpty,
    ErrorWithPair(usize, usize),
    ErrorWithString(String),
    ErrorWithStruct { a: usize, b: usize },
}

impl std::fmt::Display for TestErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            TestErrorKind::ErrorEmpty => write!(f, "empty"),
            TestErrorKind::ErrorWithPair(..) => write!(f, "pair"),
            TestErrorKind::ErrorWithString(..) => write!(f, "string"),
            TestErrorKind::ErrorWithStruct { .. } => write!(f, "struct"),
        }
    }
}

impl Detail for TestErrorKind {
    fn code(&self) -> u32 {
        match *self {
            TestErrorKind::ErrorEmpty => 1,
            TestErrorKind::ErrorWithPair(..) => 2,
            TestErrorKind::ErrorWithString(..) => 3,
            TestErrorKind::ErrorWithStruct { .. } => 4,
        }
    }
}

#[test]
fn macro_diags_with_kind() {
    let e = basic_diag!(TestErrorKind::ErrorEmpty);
    assert_eq!(e.detail().to_string(), "empty");

    let e = basic_diag!(TestErrorKind::ErrorWithPair(10, 20));
    assert_eq!(e.detail().to_string(), "pair");

    let e = basic_diag!(TestErrorKind::ErrorWithString("aaa".to_string()));
    assert_eq!(e.detail().to_string(), "string");

    let e = basic_diag!(TestErrorKind::ErrorWithStruct { a: 10, b: 20 });
    assert_eq!(e.detail().to_string(), "struct");
}

#[test]
fn macro_diags_with_kind_and_quotes() {
    let input = "line 1;\nline 2;\nline 3; // comment\nline 4;\nline 5;\nline 6;\nline 7;\nline 8;\nline 9;\nline 10;\n";
    let ref mut r = MemCharReader::with_path("src/example.txt", input.as_bytes());

    r.skip_chars(9).unwrap();
    let p1 = r.position();
    r.skip_chars(7).unwrap();
    let p2 = r.position();

    let e = parse_diag!(TestErrorKind::ErrorEmpty, r, {
        p1, p2 => "msg"
    });

    let es = e.to_string();

    assert!(es.contains("  2| line 2;\n   | ^^^^^^^ msg\n"));
}
