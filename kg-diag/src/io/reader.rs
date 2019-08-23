use std::borrow::Cow;
use std::char;
use std::path::Path;

use super::*;


pub trait Reader {
    fn path(&self) -> Option<&Path>;

    fn len(&self) -> Option<usize>;

    fn eof(&self) -> bool;

    fn position(&self) -> Position;

    fn seek(&mut self, pos: Position) -> IoResult<()>;

    fn input(&mut self) -> IoResult<Cow<str>>;

    fn slice(&mut self, start: usize, end: usize) -> IoResult<Cow<str>>;

    #[inline]
    fn slice_pos(&mut self, from: Position, to: Position) -> IoResult<Cow<str>> {
        self.slice(from.offset, to.offset)
    }

    fn reset(&mut self) -> IoResult<()> {
        self.seek(Default::default())
    }

    fn quote(
        &mut self,
        from: Position,
        to: Position,
        lines_before: u32,
        lines_after: u32,
        message: Cow<str>,
    ) -> Quote;
}

pub trait ByteReader: Reader {
    fn next_byte(&mut self) -> IoResult<Option<u8>>;

    fn peek_byte(&mut self, lookahead: usize) -> IoResult<Option<u8>>;

    fn peek_byte_pos(&mut self, lookahead: usize) -> IoResult<Option<(u8, Position)>>;

    fn skip_bytes(&mut self, skip: usize) -> IoResult<()>;
}

pub trait CharReader: Reader {
    fn next_char(&mut self) -> IoResult<Option<char>>;

    fn peek_char(&mut self, lookahead: usize) -> IoResult<Option<char>>;

    fn peek_char_pos(&mut self, lookahead: usize) -> IoResult<Option<(char, Position)>>;

    fn skip_chars(&mut self, skip: usize) -> IoResult<()>;

    fn match_str(&mut self, s: &str) -> IoResult<bool>;

    fn match_str_term(&mut self, s: &str, f: &mut dyn FnMut(Option<char>) -> bool) -> IoResult<bool>;

    fn match_char(&mut self, c: char) -> IoResult<bool> {
        if let Some(k) = self.peek_char(0)? {
            Ok(c == k)
        } else {
            Ok(false)
        }
    }

    #[inline]
    fn skip_whitespace(&mut self) -> IoResult<()> {
        while let Some(c) = self.peek_char(0)? {
            if c.is_whitespace() {
                self.next_char()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    #[inline]
    fn skip_whitespace_nonl(&mut self) -> IoResult<()> {
        while let Some(c) = self.peek_char(0)? {
            if c.is_whitespace() && c != '\n' {
                self.next_char()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    #[inline]
    fn scan(&mut self, f: &mut dyn FnMut(char) -> bool) -> IoResult<Cow<str>> {
        let s = self.position().offset;
        while let Some(c) = self.peek_char(0)? {
            if f(c) {
                self.next_char()?;
            } else {
                break;
            }
        }
        let offset = self.position().offset;
        self.slice(s, offset)
    }

    #[inline]
    fn skip_until(&mut self, f: &mut dyn FnMut(char) -> bool) -> IoResult<()> {
        while let Some(c) = self.peek_char(0)? {
            if f(c) {
                break;
            } else {
                self.next_char()?;
            }
        }
        Ok(())
    }

    #[inline]
    fn skip_while(&mut self, f: &mut dyn FnMut(char) -> bool) -> IoResult<()> {
        while let Some(c) = self.peek_char(0)? {
            if f(c) {
                self.next_char()?;
            } else {
                break;
            }
        }
        Ok(())
    }
}

fn consume_bom(input: &[u8]) -> &[u8] {
    let mut input= input;
    if input.len() >= 6 {
        if &input[..6] == "\u{EF}\u{BB}\u{BF}".as_bytes() {
            input = &input[6..input.len()];
        }
    }
    input
}

#[derive(Debug, Clone)]
pub struct MemCharReader<'a> {
    path: Option<&'a Path>,
    data: &'a [u8],
    pos: Position,
    c: char,
    len: usize,
}

impl<'a> MemCharReader<'a> {
    pub fn new(input: &'a [u8]) -> MemCharReader<'a> {
        let input = consume_bom(&input);
        MemCharReader {
            path: None,
            data: input,
            pos: Position::new(),
            c: '\0',
            len: 0,
        }
    }

    pub fn with_path<P: AsRef<Path> + ?Sized + 'a>(
        path: &'a P,
        input: &'a [u8],
    ) -> MemCharReader<'a> {
        let input = consume_bom(&input);
        MemCharReader {
            path: Some(path.as_ref()),
            data: input,
            pos: Position::new(),
            c: '\0',
            len: 0,
        }
    }

    fn encoding_err<T>(&mut self, len: usize) -> IoResult<T> {
        Err(IoErrorDetail::Utf8InvalidEncoding {
            offset: self.pos.offset,
            len,
        })
    }

    fn eof_err<T>(&mut self) -> IoResult<T> {
        Err(IoErrorDetail::Utf8UnexpectedEof {
            offset: self.pos.offset,
        })
    }

    fn next(&mut self) -> IoResult<()> {
        if self.len > 0 {
            self.pos.offset += self.len;
            if self.c == '\n' {
                self.pos.inc_line();
            } else {
                self.pos.inc_column();
            }
            self.len = 0;
        }

        unsafe {
            let len = self.data.len();
            let i = self.pos.offset;
            if i == len {
                return Ok(());
            }
            let b = *self.data.get_unchecked(i);
            if b < 0b10000000u8 {
                self.len = 1;
                self.c = char::from_u32_unchecked(b as u32);
            } else if b < 0b11000000u8 {
                return self.encoding_err(1);
            } else if b < 0b11100000u8 {
                if len < i + 1 {
                    return self.eof_err();
                }
                self.len = 2;
                let b1 = self.data.get_unchecked(i + 1);
                self.c = char::from_u32_unchecked(
                    ((b & 0b00011111u8) as u32).wrapping_shl(6) + (b1 & 0b00111111u8) as u32,
                );
            } else if b < 0b11110000u8 {
                if len < i + 2 {
                    return self.eof_err();
                }
                self.len = 3;
                let b1 = self.data.get_unchecked(i + 1);
                let b2 = self.data.get_unchecked(i + 2);
                self.c = char::from_u32_unchecked(
                    ((b & 0b00001111u8) as u32).wrapping_shl(12)
                        + ((b1 & 0b00111111u8) as u32).wrapping_shl(6)
                        + (b2 & 0b00111111u8) as u32,
                );
            } else if b <= 0b11110100u8 {
                if len < i + 3 {
                    return self.eof_err();
                }
                self.len = 4;
                let b1 = self.data.get_unchecked(i + 1);
                let b2 = self.data.get_unchecked(i + 2);
                let b3 = self.data.get_unchecked(i + 3);
                self.c = char::from_u32_unchecked(
                    ((b & 0b00000111u8) as u32).wrapping_shl(18)
                        + ((b1 & 0b00111111u8) as u32).wrapping_shl(12)
                        + ((b2 & 0b00111111) as u32).wrapping_shl(6)
                        + (b3 & 0b00111111) as u32,
                );
            } else {
                return self.encoding_err(4);
            }
        }
        Ok(())
    }
}

impl<'a> Reader for MemCharReader<'a> {
    fn path(&self) -> Option<&Path> {
        self.path
    }

    fn len(&self) -> Option<usize> {
        Some(self.data.len())
    }

    fn eof(&self) -> bool {
        self.pos.offset >= self.data.len()
    }

    fn position(&self) -> Position {
        self.pos
    }

    fn seek(&mut self, pos: Position) -> IoResult<()> {
        self.pos = pos;
        self.c = '\0';
        self.len = 0;
        Ok(())
    }

    /// will panic in debug if slice is not a valid utf8
    #[cfg(debug_assertions)]
    fn input(&mut self) -> IoResult<Cow<str>> {
        Ok(Cow::Borrowed(
            std::str::from_utf8(&self.data).expect("input must be a valid utf8"),
        ))
    }

    #[cfg(not(debug_assertions))]
    fn input(&mut self) -> IoResult<Cow<str>> {
        Ok(Cow::Borrowed(unsafe {
            std::str::from_utf8_unchecked(&self.data)
        }))
    }

    /// will panic in debug if slice is not a valid utf8
    #[cfg(debug_assertions)]
    fn slice(&mut self, start: usize, end: usize) -> IoResult<Cow<str>> {
        Ok(Cow::Borrowed(
            std::str::from_utf8(&self.data[start..end]).expect("slice must be a valid utf8"),
        ))
    }

    #[cfg(not(debug_assertions))]
    fn slice(&mut self, start: usize, end: usize) -> IoResult<Cow<str>> {
        Ok(Cow::Borrowed(unsafe {
            std::str::from_utf8_unchecked(&self.data[start..end])
        }))
    }

    fn quote(
        &mut self,
        from: Position,
        to: Position,
        lines_before: u32,
        lines_after: u32,
        message: Cow<str>,
    ) -> Quote {
        Quote::new(
            self.path,
            self.data,
            from,
            to,
            lines_before,
            lines_after,
            message,
        )
    }
}

impl<'a> CharReader for MemCharReader<'a> {
    fn next_char(&mut self) -> IoResult<Option<char>> {
        self.next()?;
        if self.len > 0 {
            Ok(Some(self.c))
        } else {
            Ok(None)
        }
    }

    fn peek_char(&mut self, lookahead: usize) -> IoResult<Option<char>> {
        if lookahead == 0 {
            if self.len == 0 {
                self.next_char()
            } else {
                Ok(Some(self.c))
            }
        } else {
            let mut r = self.clone();
            for _ in 0..lookahead {
                if let None = r.next_char()? {
                    return Ok(None);
                }
            }
            Ok(Some(r.c))
        }
    }

    fn peek_char_pos(&mut self, lookahead: usize) -> IoResult<Option<(char, Position)>> {
        if lookahead == 0 {
            if self.len == 0 {
                self.next_char().map(|c| c.map(|c| (c, self.position())))
            } else {
                return Ok(Some((self.c, self.pos)));
            }
        } else {
            let mut r = self.clone();
            for _ in 0..lookahead {
                if let None = r.next_char()? {
                    return Ok(None);
                }
            }
            Ok(Some((r.c, r.pos)))
        }
    }

    fn skip_chars(&mut self, skip: usize) -> IoResult<()> {
        for _ in 0..skip {
            self.next_char()?;
        }
        Ok(())
    }

    fn match_str(&mut self, s: &str) -> IoResult<bool> {
        if s.len() > self.data.len() - self.pos.offset {
            Ok(false)
        } else {
            let d = &self.data[self.pos.offset..self.pos.offset + s.len()];
            Ok(d == s.as_bytes())
        }
    }

    #[inline]
    fn match_str_term(&mut self, s: &str, f: &mut dyn FnMut(Option<char>) -> bool) -> IoResult<bool> {
        let mut r = self.clone();
        if r.match_str(s)? {
            let e = self.pos.offset + s.len();
            while r.pos.offset < e {
                r.next_char()?;
            }
            Ok(f(r.peek_char(0)?))
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemByteReader<'a> {
    path: Option<&'a Path>,
    data: &'a [u8],
    pos: Position,
    left: usize,
}

impl<'a> MemByteReader<'a> {
    pub fn new(input: &'a [u8]) -> MemByteReader<'a> {
        MemByteReader {
            path: None,
            data: input,
            pos: Position::new(),
            left: 0,
        }
    }

    pub fn with_path(path: &'a Path, input: &'a [u8]) -> MemByteReader<'a> {
        MemByteReader {
            path: Some(path),
            data: input,
            pos: Position::new(),
            left: 0,
        }
    }

    fn encoding_err<T>(&mut self, len: usize) -> IoResult<T> {
        Err(IoErrorDetail::Utf8InvalidEncoding {
            offset: self.pos.offset,
            len,
        })
    }

    fn eof_err<T>(&mut self) -> IoResult<T> {
        Err(IoErrorDetail::Utf8UnexpectedEof {
            offset: self.pos.offset,
        })
    }
}

impl<'a> Reader for MemByteReader<'a> {
    fn path(&self) -> Option<&Path> {
        self.path
    }

    fn len(&self) -> Option<usize> {
        Some(self.data.len())
    }

    fn eof(&self) -> bool {
        self.pos.offset >= self.data.len()
    }

    fn position(&self) -> Position {
        self.pos
    }

    fn seek(&mut self, pos: Position) -> IoResult<()> {
        self.pos = pos;
        Ok(())
    }

    /// will panic in debug if slice is not a valid utf8
    #[cfg(debug_assertions)]
    fn input(&mut self) -> IoResult<Cow<str>> {
        Ok(Cow::Borrowed(
            std::str::from_utf8(&self.data).expect("input must be a valid utf8"),
        ))
    }

    #[cfg(not(debug_assertions))]
    fn input(&mut self) -> IoResult<Cow<str>> {
        Ok(Cow::Borrowed(unsafe {
            std::str::from_utf8_unchecked(&self.data)
        }))
    }

    /// will panic in debug if slice is not a valid utf8
    #[cfg(debug_assertions)]
    fn slice(&mut self, start: usize, end: usize) -> IoResult<Cow<str>> {
        Ok(Cow::Borrowed(
            std::str::from_utf8(&self.data[start..end]).expect("slice must be a valid utf8"),
        ))
    }

    #[cfg(not(debug_assertions))]
    fn slice(&mut self, start: usize, end: usize) -> IoResult<Cow<str>> {
        Ok(Cow::Borrowed(unsafe {
            std::str::from_utf8_unchecked(&self.data[start..end])
        }))
    }

    fn quote(
        &mut self,
        from: Position,
        to: Position,
        lines_before: u32,
        lines_after: u32,
        message: Cow<str>,
    ) -> Quote {
        Quote::new(
            self.path,
            self.data,
            from,
            to,
            lines_before,
            lines_after,
            message,
        )
    }
}

impl<'a> ByteReader for MemByteReader<'a> {
    fn next_byte(&mut self) -> IoResult<Option<u8>> {
        if self.pos.offset < self.data.len() {
            unsafe {
                let off = self.pos.offset;
                self.pos.offset += 1;
                let b = *self.data.get_unchecked(off);
                if self.left == 0 {
                    if b == b'\n' {
                        self.left = 0;
                        self.pos.inc_line();
                    } else if b < 0b10000000 {
                        self.left = 0;
                        self.pos.inc_column();
                    } else if b < 0b11000000 {
                        return self.encoding_err(1);
                    } else if b < 0b11100000 {
                        self.left = 1;
                    } else if b < 0b11110000 {
                        self.left = 2;
                    } else if b <= 0b11110100 {
                        self.left = 3;
                    } else {
                        return self.encoding_err(3);
                    }
                } else if b >= 0b11000000 {
                    self.left -= 1;
                } else {
                    return self.encoding_err(self.left);
                }
                Ok(Some(b))
            }
        } else if self.left > 0 {
            return self.eof_err();
        } else {
            Ok(None)
        }
    }

    fn peek_byte(&mut self, lookahead: usize) -> IoResult<Option<u8>> {
        let offset = self.pos.offset + lookahead;
        if offset < self.data.len() {
            unsafe { Ok(Some(*self.data.get_unchecked(offset))) }
        } else {
            Ok(None)
        }
    }

    fn peek_byte_pos(&mut self, lookahead: usize) -> IoResult<Option<(u8, Position)>> {
        if lookahead == 0 {
            if self.pos.offset < self.data.len() {
                unsafe { Ok(Some((*self.data.get_unchecked(self.pos.offset), self.pos))) }
            } else {
                Ok(None)
            }
        } else {
            let mut r = self.clone();
            for _ in 0..lookahead {
                if let None = r.next_byte()? {
                    return Ok(None);
                }
            }
            unsafe { Ok(Some((*r.data.get_unchecked(r.pos.offset), r.pos))) }
        }
    }

    fn skip_bytes(&mut self, skip: usize) -> IoResult<()> {
        for _ in 0..skip {
            self.next_byte()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_reader_match_str_term() {
        let mut r = MemCharReader::new("example input".as_bytes());
        let m = r
            .match_str_term("example", &mut |c| c.is_none() || c.unwrap().is_whitespace())
            .unwrap();
        assert!(m);
        assert_eq!(r.position().offset, 0);
    }

    mod char_reader_errors {
        use super::*;

        #[test]
        fn utf8_encoding_err_offset() {
            let bytes: &[u8] = &[0x41, 0x42, 0xff];
            let mut r = MemCharReader::new(bytes);
            r.skip_chars(2).unwrap();

            let err = r.next_char().expect_err("Error expected");

            match err {
                IoErrorDetail::Utf8InvalidEncoding { offset, len } => {
                    assert_eq!(offset, 2);
                    assert_eq!(len, 4);
                }
                _ => panic!("wrong detail in error"),
            }
        }
    }

    #[test]
    fn char_reader_diacritics() {
        let input = "老aąćżńęóź";
        let mut r = MemCharReader::new(input.as_bytes());

        assert_eq!(r.next_char().unwrap().unwrap(), '老');
        assert_eq!(r.next_char().unwrap().unwrap(), 'a');
        assert_eq!(r.next_char().unwrap().unwrap(), 'ą');
        assert_eq!(r.next_char().unwrap().unwrap(), 'ć');
        assert_eq!(r.next_char().unwrap().unwrap(), 'ż');
        assert_eq!(r.next_char().unwrap().unwrap(), 'ń');
        assert_eq!(r.next_char().unwrap().unwrap(), 'ę');
        assert_eq!(r.next_char().unwrap().unwrap(), 'ó');
        assert_eq!(r.next_char().unwrap().unwrap(), 'ź');
    }
}
