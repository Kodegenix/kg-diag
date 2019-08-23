use super::*;

const PARSE_TASK_NAME: &str = "paring a number literal";


#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Notation {
    #[display("d")]
    Decimal,
    #[display("f")]
    Float,
    #[display("e")]
    Exponent,
    #[display("o")]
    Octal,
    #[display("x")]
    Hex,
    #[display("b")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[display("{sign}{notation}")]
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


impl LexTerm for Number {}


fn parse_simple_num<N: NotationConfig>(n: &N,
                                       sign: Sign,
                                       r: &mut dyn CharReader) -> ParseResult<LexToken<Number>> {
    let p1 = r.position();
    let mut p = p1;

    if sign == Sign::None || (sign == Sign::Minus && n.allow_minus()) || (sign == Sign::Plus && n.allow_plus()) {
        if sign != Sign::None {
            r.skip_chars(1)?;
        }
        r.skip_chars(n.prefix().len())?;
        p = r.position();
        if !n.allow_underscores() {
            r.skip_while(&mut |c| n.is_digit(c))?;
        } else {
            let mut digit = false;
            while let Some(c) = r.peek_char(0)? {
                if c == '_' {
                    if !digit {
                        break;
                    }
                } else if n.is_digit(c) {
                    digit = true;
                } else {
                    break;
                }
                r.next_char()?;
            }
        }
    }

    let p2 = r.position();
    if p2 > p {
        Ok(LexToken::new(Number::new(n.get_notation(), sign), p1, p2))
    } else {
        Err(match r.peek_char(0)? {
            Some(c) => ParseErrorDetail::UnexpectedInput {
                pos: p2,
                found: Some(Input::Char(c)),
                expected: Some(n.get_expected_digit()),
                task: n.get_task_name().into(),
            },
            None => ParseErrorDetail::UnexpectedEof {
                pos: p2,
                expected: Some(n.get_expected_digit()),
                task: n.get_task_name().into(),
            }
        })
    }
}


pub struct NumberParser {
    pub decimal: DecimalConfig,
    pub hex: HexConfig,
    pub octal: OctalConfig,
    pub binary: BinaryConfig,
}

impl NumberParser {
    pub fn new() -> NumberParser {
        NumberParser {
            decimal: DecimalConfig::new(),
            hex: HexConfig::new(),
            octal: OctalConfig::new(),
            binary: BinaryConfig::new(),
        }
    }

    pub fn is_at_start(&self, r: &mut dyn CharReader) -> IoResult<bool> {
        Ok(self.hex.is_at_start(r)?
            || self.octal.is_at_start(r)?
            || self.binary.is_at_start(r)?
            || self.decimal.is_at_start(r)?)
    }

    pub fn parse_number(&self, r: &mut dyn CharReader) -> ParseResult<LexToken<Number>> {
        let mut sign = Sign::None;
        if let Some(c) = r.peek_char(0)? {
            if c == '-' {
                sign = Sign::Minus;
            } else if c == '+' {
                sign = Sign::Plus;
            }
        } else {
            return Err(ParseErrorDetail::UnexpectedEof {
                pos: r.position(),
                expected: None,
                task: PARSE_TASK_NAME.into(),
            });
        }

        if self.hex.is_at_start(r)? {
            self.parse_hex(sign, r)
        } else if self.octal.is_at_start(r)? {
            self.parse_octal(sign, r)
        } else if self.binary.is_at_start(r)? {
            self.parse_binary(sign, r)
        } else if self.decimal.is_at_start(r)? {
            self.parse_decimal(sign, r)
        } else {
            Err(match r.peek_char(0)? {
                Some(c) => ParseErrorDetail::UnexpectedInput {
                    pos: r.position(),
                    found: Some(Input::Char(c)),
                    expected: None,
                    task: PARSE_TASK_NAME.into(),
                },
                None => ParseErrorDetail::UnexpectedEof {
                    pos: r.position(),
                    expected: None,
                    task: PARSE_TASK_NAME.into(),
                }
            })
        }
    }

    fn parse_hex(&self, sign: Sign, r: &mut dyn CharReader) -> ParseResult<LexToken<Number>> {
        parse_simple_num(&self.hex, sign, r)
    }

    fn parse_octal(&self, sign: Sign, r: &mut dyn CharReader) -> ParseResult<LexToken<Number>> {
        parse_simple_num(&self.octal, sign, r)
    }

    fn parse_binary(&self, sign: Sign, r: &mut dyn CharReader) -> ParseResult<LexToken<Number>> {
        parse_simple_num(&self.binary, sign, r)
    }

    fn parse_decimal(&self, sign: Sign, r: &mut dyn CharReader) -> ParseResult<LexToken<Number>> {
        let p1 = r.position();

        let mut notation = None;
        let mut last = ' ';

        if sign == Sign::None || (sign == Sign::Minus && self.decimal.allow_minus) || (sign == Sign::Plus && self.decimal.allow_plus) {
            if sign != Sign::None {
                r.skip_chars(1)?;
            }

            while let Some(c) = r.peek_char(0)? {
                if self.decimal.is_digit(c) {
                    match last {
                        ' ' => notation = Some(Notation::Decimal),
                        '.' => notation = Some(Notation::Float),
                        'e' | '-' => notation = Some(Notation::Exponent),
                        '0' => {}
                        _ => unreachable!(),
                    }
                    last = '0';
                } else if c == '_' && self.decimal.allow_underscores && (last == '0' || last == 'e' || last == '-') {
                    // skip
                } else if c == '.' && self.decimal.allow_float && last == '0' && notation == Some(Notation::Decimal) {
                    last = '.';
                } else if ((c == 'e' && self.decimal.case != Case::Upper)
                        || (c == 'E' && self.decimal.case != Case::Lower))
                        && self.decimal.allow_exponent && last == '0' && notation != Some(Notation::Exponent) {
                    last = 'e';
                } else if (c == '-' || c == '+') && last == 'e' {
                    last = '-';
                } else {
                    break;
                }
                r.next_char()?;
            }
        }

        let p2 = r.position();

        if notation.is_some() && last == '0' {
            Ok(LexToken::new(Number::new(notation.unwrap(), sign), p1, p2))
        } else {
            let expected = match last {
                ' ' | '.' => {
                    let mut expected = Vec::new();
                    if self.decimal.allow_minus {
                        expected.push(Expected::Char('-'));
                    }
                    if self.decimal.allow_plus {
                        expected.push(Expected::Char('+'));
                    }
                    expected.push(Expected::CharRange('0', '9'));
                    Expected::one_of(expected)
                },
                'e' => if self.decimal.allow_underscores {
                    Expected::one_of(vec![Expected::CharRange('0', '9'), Expected::Char('_'), Expected::Char('-'), Expected::Char('+')])
                } else {
                    Expected::one_of(vec![Expected::CharRange('0', '9'), Expected::Char('-'), Expected::Char('+')])
                },
                '-' => if self.decimal.allow_underscores {
                    Expected::one_of(vec![Expected::CharRange('0', '9'), Expected::Char('_')])
                } else {
                    Expected::CharRange('0', '9')
                },
                _ => unreachable!(),
            };

            Err(match r.peek_char(0)? {
                Some(c) => ParseErrorDetail::UnexpectedInput {
                    pos: p2,
                    found: Some(Input::Char(c)),
                    expected: Some(expected),
                    task: self.decimal.get_task_name().into(),
                },
                None => ParseErrorDetail::UnexpectedEof {
                    pos: p2,
                    expected: Some(expected),
                    task: self.decimal.get_task_name().into(),
                }
            })
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    Any,
    Upper,
    Lower,
}


trait NotationConfig {
    fn is_enabled(&self) -> bool;

    fn allow_plus(&self) -> bool;

    fn allow_minus(&self) -> bool;

    fn allow_underscores(&self) -> bool;

    fn prefix(&self) -> &str {
        ""
    }

    fn case(&self) -> Case {
        Case::Any
    }

    fn is_at_start(&self, r: &mut dyn CharReader) -> IoResult<bool> {
        Ok(if self.is_enabled() {
            if let Some(c) = r.peek_char(0)? {
                if (c == '-' && self.allow_minus()) || (c == '+' && self.allow_plus()) {
                    true
                } else if self.prefix().is_empty() {
                    self.is_digit(c)
                } else {
                    r.match_str(self.prefix())?
                }
            } else {
                false
            }
        } else {
            false
        })
    }

    fn is_digit(&self, c: char) -> bool;

    fn get_notation(&self) -> Notation;

    fn get_expected_digit(&self) -> Expected;

    fn get_task_name(&self) -> &str {
        PARSE_TASK_NAME
    }
}


pub struct DecimalConfig {
    pub enabled: bool,
    pub allow_minus: bool,
    pub allow_plus: bool,
    pub allow_underscores: bool,
    pub allow_float: bool,
    pub allow_exponent: bool,
    pub case: Case,
}

impl DecimalConfig {
    fn new() -> DecimalConfig {
        DecimalConfig {
            enabled: true,
            allow_minus: true,
            allow_plus: true,
            allow_underscores: true,
            allow_float: true,
            allow_exponent: true,
            case: Case::Any,
        }
    }
}


impl NotationConfig for DecimalConfig {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn allow_plus(&self) -> bool {
        self.allow_plus
    }

    fn allow_minus(&self) -> bool {
        self.allow_minus
    }

    fn allow_underscores(&self) -> bool {
        self.allow_underscores
    }

    fn case(&self) -> Case {
        self.case
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn get_notation(&self) -> Notation {
        Notation::Decimal
    }

    fn get_expected_digit(&self) -> Expected {
        Expected::CharRange('0', '9')
    }

    fn get_task_name(&self) -> &str {
        "parsing a decimal number literal"
    }
}


pub struct HexConfig {
    pub enabled: bool,
    pub allow_minus: bool,
    pub allow_plus: bool,
    pub allow_underscores: bool,
    pub prefix: String,
    pub case: Case,
}

impl HexConfig {
    fn new() -> HexConfig {
        HexConfig {
            enabled: true,
            allow_minus: true,
            allow_plus: true,
            allow_underscores: true,
            prefix: String::from("0x"),
            case: Case::Any,
        }
    }
}

impl NotationConfig for HexConfig {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn allow_plus(&self) -> bool {
        self.allow_plus
    }

    fn allow_minus(&self) -> bool {
        self.allow_minus
    }

    fn allow_underscores(&self) -> bool {
        self.allow_underscores
    }

    fn prefix(&self) -> &str {
        &self.prefix
    }

    fn case(&self) -> Case {
        self.case
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9' || match self.case {
            Case::Any => (c >= 'A' && c <= 'F') || (c >= 'a' && c <= 'f'),
            Case::Upper => c >= 'A' && c <= 'F',
            Case::Lower => c >= 'a' && c <= 'f',
        }
    }

    fn get_notation(&self) -> Notation {
        Notation::Hex
    }

    fn get_expected_digit(&self) -> Expected {
        match self.case {
            Case::Any => Expected::OneOf(vec![Expected::CharRange('0', '9'), Expected::CharRange('A', 'F'), Expected::CharRange('a', 'f')]),
            Case::Lower => Expected::OneOf(vec![Expected::CharRange('0', '9'), Expected::CharRange('a', 'f')]),
            Case::Upper => Expected::OneOf(vec![Expected::CharRange('0', '9'), Expected::CharRange('A', 'F')]),
        }
    }

    fn get_task_name(&self) -> &str {
        "parsing a hexadecimal number literal"
    }
}


pub struct OctalConfig {
    pub enabled: bool,
    pub allow_minus: bool,
    pub allow_plus: bool,
    pub allow_underscores: bool,
    pub prefix: String,
}

impl OctalConfig {
    fn new() -> OctalConfig {
        OctalConfig {
            enabled: true,
            allow_minus: true,
            allow_plus: true,
            allow_underscores: true,
            prefix: String::from("0o"),
        }
    }
}

impl NotationConfig for OctalConfig {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn allow_plus(&self) -> bool {
        self.allow_plus
    }

    fn allow_minus(&self) -> bool {
        self.allow_minus
    }

    fn allow_underscores(&self) -> bool {
        self.allow_underscores
    }

    fn prefix(&self) -> &str {
        &self.prefix
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '7'
    }

    fn get_notation(&self) -> Notation {
        Notation::Octal
    }

    fn get_expected_digit(&self) -> Expected {
        Expected::CharRange('0', '7')
    }

    fn get_task_name(&self) -> &str {
        "parsing an octal number literal"
    }
}


pub struct BinaryConfig {
    pub enabled: bool,
    pub allow_minus: bool,
    pub allow_plus: bool,
    pub allow_underscores: bool,
    pub prefix: String,
}

impl BinaryConfig {
    fn new() -> BinaryConfig {
        BinaryConfig {
            enabled: true,
            allow_minus: true,
            allow_plus: true,
            allow_underscores: true,
            prefix: String::from("0b"),
        }
    }
}

impl NotationConfig for BinaryConfig {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn allow_plus(&self) -> bool {
        self.allow_plus
    }

    fn allow_minus(&self) -> bool {
        self.allow_minus
    }

    fn allow_underscores(&self) -> bool {
        self.allow_underscores
    }

    fn prefix(&self) -> &str {
        &self.prefix
    }

    fn is_digit(&self, c: char) -> bool {
        c == '0' || c == '1'
    }

    fn get_notation(&self) -> Notation {
        Notation::Binary
    }

    fn get_expected_digit(&self) -> Expected {
        Expected::CharRange('0', '1')
    }

    fn get_task_name(&self) -> &str {
        "parsing a binary number literal"
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_decimal() {
        let np = NumberParser::new();

        let mut r = MemCharReader::new(b"- ");

        match np.parse_number(&mut r) {
            Ok(n) => println!("{} {:?}", n, r.slice_pos(n.from(), n.to()).unwrap()),
            Err(err) => println!("{}", err),
        }
    }
}