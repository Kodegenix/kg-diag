use super::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notation {
    Decimal,
    Float,
    Exponent,
    Octal,
    Hex,
    Binary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Sign {
    #[display("")]
    None,
    #[display("-")]
    Minus,
    #[display("+")]
    Plus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Number {
    notation: Notation,
    sign: Sign,
}

impl Number {
    pub fn new(notation: Notation, sign: Sign) -> Number {
        Number {
            notation,
            sign,
        }
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

impl LexTerm for Number {}

pub struct NumberParser {
    decimal: DecimalConfig,
    hex: HexConfig,
    octal: OctalConfig,
    binary: BinaryConfig,
}

impl NumberParser {
    fn new() -> NumberParser {
        NumberParser {
            decimal: DecimalConfig::new(),
            hex: HexConfig::new(),
            octal: OctalConfig::new(),
            binary: BinaryConfig::new(),
        }
    }

    pub fn is_at_start(&self, r: &mut dyn CharReader) -> IoResult<bool> {
        Ok(self.decimal.is_at_start(r)?
            || self.hex.is_at_start(r)?
            || self.octal.is_at_start(r)?
            || self.binary.is_at_start(r)?)
    }

    pub fn parse_number(&mut self, r: &mut dyn CharReader) -> IoResult<LexToken<Number>> {
        let mut sign = Sign::None;
        if let Some(c) = r.peek_char(0)? {
            if c == '-' {
                sign = Sign::Minus;
            } else if c == '+' {
                sign = Sign::Plus;
            }
        } else {
            return Err(IoErrorDetail::UnexpectedEof {
                pos: r.position(),
                task: "parsing a number literal".into(),
            });
        }

        if self.hex.is_at_start(r)? {
            self.parse_hex(sign, r)
        } else if self.octal.is_at_start(r)? {
            self.parse_octal(sign, r)
        } else if self.binary.is_at_start(r)? {
            self.parse_binary(sign, r)
        } else {
            self.parse_decimal(sign, r)
        }
    }

    fn parse_hex(&mut self, sign: Sign, r: &mut dyn CharReader) -> IoResult<LexToken<Number>> {
        let p1 = r.position();
        if (sign == Sign::Minus && !self.hex.allow_minus) || (sign == Sign::Plus && !self.hex.allow_plus) {
            return Err(IoErrorDetail::UnexpectedInput {
                pos: r.position(),
                found: sign.to_string(),
                expected: vec![String::from("an hexadecimal digit")],
                task: "parsing hexadecimal number literal".into(),
            });
        } else {
            r.skip_chars(1)?;
        }
        r.skip_chars(self.hex.prefix.len())?;
        if !self.hex.allow_underscores {
            r.skip_while(&mut |c| self.hex.is_digit(c))?;
        } else {
            let mut digit = false;
            while let Some(c) = r.peek_char(0)? {
                if c == '_' {
                    if !digit {
                        return Err(IoErrorDetail::UnexpectedInput {
                            pos: r.position(),
                            found: c.to_string(),
                            expected: vec![String::from("an hexadecimal digit")],
                            task: "parsing hexadecimal number literal".into(),
                        });
                    }
                    digit = false;
                } else if self.hex.is_digit(c) {
                    digit = true;
                } else {
                    break;
                }
                r.next_char()?;
            }
        }
        let p2 = r.position();
        Ok(LexToken::new(Number::new(Notation::Hex, sign), p1, p2))
    }

    fn parse_octal(&mut self, sign: Sign, r: &mut dyn CharReader) -> IoResult<LexToken<Number>> {
        let p1 = r.position();
        if (sign == Sign::Minus && !self.octal.allow_minus) || (sign == Sign::Plus && !self.octal.allow_plus) {
            return Err(IoErrorDetail::UnexpectedInput {
                pos: r.position(),
                found: sign.to_string(),
                expected: vec![String::from("an octal digit")],
                task: "parsing octal number literal".into(),
            });
        } else {
            r.skip_chars(1)?;
        }
        r.skip_chars(self.octal.prefix.len())?;
        if !self.octal.allow_underscores {
            r.skip_while(&mut |c| self.octal.is_digit(c))?;
        } else {
            let mut digit = false;
            while let Some(c) = r.peek_char(0)? {
                if c == '_' {
                    if !digit {
                        return Err(IoErrorDetail::UnexpectedInput {
                            pos: r.position(),
                            found: c.to_string(),
                            expected: vec![String::from("an octal digit")],
                            task: "parsing octal number literal".into(),
                        });
                    }
                    digit = false;
                } else if self.octal.is_digit(c) {
                    digit = true;
                } else {
                    break;
                }
                r.next_char()?;
            }
        }
        let p2 = r.position();
        Ok(LexToken::new(Number::new(Notation::Octal, sign), p1, p2))
    }

    fn parse_binary(&mut self, sign: Sign, r: &mut dyn CharReader) -> IoResult<LexToken<Number>> {
        let p1 = r.position();
        if (sign == Sign::Minus && !self.binary.allow_minus) || (sign == Sign::Plus && !self.binary.allow_plus) {
            return Err(IoErrorDetail::UnexpectedInput {
                pos: r.position(),
                found: sign.to_string(),
                expected: vec![String::from("'0'"), String::from("'1'")],
                task: "parsing binary number literal".into(),
            });
        } else {
            r.skip_chars(1)?;
        }
        r.skip_chars(self.binary.prefix.len())?;
        if !self.binary.allow_underscores {
            r.skip_while(&mut |c| self.binary.is_digit(c))?;
        } else {
            let mut digit = false;
            while let Some(c) = r.peek_char(0)? {
                if c == '_' {
                    if !digit {
                        return Err(IoErrorDetail::UnexpectedInput {
                            pos: r.position(),
                            found: c.to_string(),
                            expected: vec![String::from("'0'"), String::from("'1'")],
                            task: "parsing binary number literal".into(),
                        });
                    }
                    digit = false;
                } else if self.binary.is_digit(c) {
                    digit = true;
                } else {
                    break;
                }
                r.next_char()?;
            }
        }
        let p2 = r.position();
        Ok(LexToken::new(Number::new(Notation::Binary, sign), p1, p2))
    }

    fn parse_decimal(&mut self, sign: Sign, r: &mut dyn CharReader) -> IoResult<LexToken<Number>> {
        let p1 = r.position();
        if (sign == Sign::Minus && !self.decimal.allow_minus) || (sign == Sign::Plus && !self.decimal.allow_plus) {
            return Err(IoErrorDetail::UnexpectedInput {
                pos: r.position(),
                found: sign.to_string(),
                expected: vec![String::from("decimal digit")],
                task: "parsing a number literal".into(),
            });
        } else {
            r.skip_chars(1)?;
        }

        let mut digit = false;
        let mut dot = false;
        while let Some(c) = r.peek_char(0)? {
            if c == '_' {
                if !digit {
                    return Err(IoErrorDetail::UnexpectedInput {
                        pos: r.position(),
                        found: c.to_string(),
                        expected: vec![String::from("decimal digit")],
                        task: "parsing a number literal".into(),
                    });
                } else if !self.decimal.allow_underscores {
                    break;
                }
                digit = false;
            } else if self.decimal.is_digit(c) {
                digit = true;
            } else if c == '.' {

            } else {
                break;
            }
            r.next_char()?;
        }

        let p2 = r.position();
        Ok(LexToken::new(Number::new(Notation::Decimal, sign), p1, p2))
    }
/*
    fn parse_float(&mut self, sign: Sign, r: &mut dyn CharReader) -> IoResult<LexToken<Number>> {
        let p1 = r.position();
        if (sign == Sign::Minus && !self.float.allow_minus) || (sign == Sign::Plus && !self.float.allow_plus) {
            return Err(IoErrorDetail::UnexpectedInput {
                pos: r.position(),
                found: sign.to_string(),
                expected: vec![String::from("decimal digit")],
                task: "parsing a number literal".into(),
            });
        } else {
            r.skip_chars(1)?;
        }

        let mut dot = false;
        let mut digit = false;
        while let Some(c) = r.peek_char(0)? {
            if c == '_' {
                if !digit {
                    return Err(IoErrorDetail::UnexpectedInput {
                        pos: r.position(),
                        found: c.to_string(),
                        expected: vec![String::from("decimal digit")],
                        task: "parsing a number literal".into(),
                    });
                } else if !self.float.allow_underscores {
                    break;
                }
                digit = false;
            } else if self.float.is_digit(c) {
                digit = true;
            } else if !dot && c == '.' {
                dot = true;
            } else if (c == 'e' && self.float.case != Case::Upper) || (c == 'E' && self.float.case != Case::Lower) {
                self.parse_exponent(r)?;
                break;
            } else {
                break;
            }
        }

        let p2 = r.position();
        Ok(LexToken::new(Number::new(Notation::Float, sign), p1, p2))
    }

    fn parse_exponent(&mut self, r: &mut dyn CharReader) -> IoResult<()> {
        r.next_char()?;
        let c = r.peek_char(0)?;
        match c {
            Some('-') | Some('+') => {
                r.skip_chars(1)?;
                let mut has_digits = false;
                r.skip_while(&mut |c| {
                    if self.float.is_digit(c) {
                        has_digits = true;
                        true
                    } else {
                        false
                    }
                })?;
                if has_digits {
                    Ok(())
                } else {
                    Err(IoErrorDetail::UnexpectedInput {
                        pos: r.position(),
                        found: c.unwrap().to_string(),
                        expected: vec![String::from("[0-9]")],
                        task: "parsing a number literal".into(),
                    })
                }
            }
            Some(c) if self.float.is_digit(c) => {
                r.skip_chars(1)?;
                r.skip_while(&mut |c| self.float.is_digit(c))?;
                Ok(())
            }
            _ => Err(IoErrorDetail::UnexpectedInput {
                pos: r.position(),
                found: c.unwrap().to_string(),
                expected: vec![String::from("'-'"), String::from("'+'"), String::from("[0-9]")],
                task: "parsing a number literal".into(),
            })
        }
    }*/
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    Any,
    Upper,
    Lower,
}


pub struct DecimalConfig {
    enabled: bool,
    allow_minus: bool,
    allow_plus: bool,
    allow_underscores: bool,
    allow_float: bool,
    allow_exponent: bool,
    case: Case,
}

impl DecimalConfig {
    fn new() -> DecimalConfig {
        DecimalConfig {
            enabled: false,
            allow_minus: false,
            allow_plus: false,
            allow_underscores: false,
            allow_float: false,
            allow_exponent: false,
            case: Case::Any,
        }
    }

    fn is_at_start(&self, r: &mut dyn CharReader) -> IoResult<bool> {
        Ok(if self.enabled {
            if let Some(c) = r.peek_char(0)? {
                self.is_digit(c)
                    || (c == '-' && self.allow_minus)
                    || (c == '+' && self.allow_plus)
            } else {
                false
            }
        } else {
            false
        })
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }
}


pub struct HexConfig {
    enabled: bool,
    allow_minus: bool,
    allow_plus: bool,
    allow_underscores: bool,
    prefix: String,
    case: Case,
}

impl HexConfig {
    fn new() -> HexConfig {
        HexConfig {
            enabled: false,
            allow_minus: false,
            allow_plus: false,
            allow_underscores: false,
            prefix: String::from("0x"),
            case: Case::Any,
        }
    }

    fn is_at_start(&self, r: &mut dyn CharReader) -> IoResult<bool> {
        Ok(if self.enabled {
            if let Some(c) = r.peek_char(0)? {
                (!self.prefix.is_empty() && r.match_str(&self.prefix)?)
                    || self.is_digit(c)
                    || (c == '-' && self.allow_minus)
                    || (c == '+' && self.allow_plus)
            } else {
                false
            }
        } else {
            false
        })
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9' || match self.case {
            Case::Any => (c >= 'A' && c <= 'F') || (c >= 'a' && c <= 'f'),
            Case::Upper => c >= 'A' && c <= 'F',
            Case::Lower => c >= 'a' && c <= 'f',
        }
    }
}


pub struct OctalConfig {
    enabled: bool,
    allow_minus: bool,
    allow_plus: bool,
    allow_underscores: bool,
    prefix: String,
}

impl OctalConfig {
    fn new() -> OctalConfig {
        OctalConfig {
            enabled: false,
            allow_minus: false,
            allow_plus: false,
            allow_underscores: false,
            prefix: String::from("0o"),
        }
    }

    fn is_at_start(&self, r: &mut dyn CharReader) -> IoResult<bool> {
        Ok(if self.enabled {
            if let Some(c) = r.peek_char(0)? {
                (!self.prefix.is_empty() && r.match_str(&self.prefix)?)
                    || self.is_digit(c)
                    || (c == '-' && self.allow_minus)
                    || (c == '+' && self.allow_plus)
            } else {
                false
            }
        } else {
            false
        })
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '7'
    }
}


pub struct BinaryConfig {
    enabled: bool,
    allow_minus: bool,
    allow_plus: bool,
    allow_underscores: bool,
    prefix: String,
}

impl BinaryConfig {
    fn new() -> BinaryConfig {
        BinaryConfig {
            enabled: false,
            allow_minus: false,
            allow_plus: false,
            allow_underscores: false,
            prefix: String::from("0b"),
        }
    }

    fn is_at_start(&self, r: &mut dyn CharReader) -> IoResult<bool> {
        Ok(if self.enabled {
            if let Some(c) = r.peek_char(0)? {
                (!self.prefix.is_empty() && r.match_str(&self.prefix)?)
                    || self.is_digit(c)
                    || (c == '-' && self.allow_minus)
                    || (c == '+' && self.allow_plus)
            } else {
                false
            }
        } else {
            false
        })
    }

    fn is_digit(&self, c: char) -> bool {
        c == '0' || c == '1'
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_decimal() {
        let mut np = NumberParser::new();

        np.decimal.enabled = false;
        np.decimal.allow_underscores = true;
        np.decimal.allow_float = true;
        np.decimal.allow_exponent = true;
      
        let mut r = MemCharReader::new(b"1234_5678.12");

        let n = np.parse_number(&mut r).unwrap();

        println!("{:?}", r.slice_pos(n.from(), n.to()).unwrap());

    }
}