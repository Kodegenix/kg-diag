use super::*;

pub mod error;
pub mod num;

pub use self::error::*;
pub use self::num::*;

pub type ParseResult<T> = Result<T, ParseErrorDetail>;
