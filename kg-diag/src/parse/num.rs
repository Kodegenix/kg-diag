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

impl Notation {
    #[inline]
    pub fn radix(&self) -> u32 {
        match *self {
            Notation::Decimal | Notation::Float | Notation::Exponent => 10,
            Notation::Hex => 16,
            Notation::Octal => 8,
            Notation::Binary => 2,
        }
    }
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

impl Sign {
    #[inline]
    fn len(&self) -> usize {
        match *self {
            Sign::None => 0,
            _ => 1,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[display("{sign}{notation}")]
pub struct Number {
    sign: Sign,
    notation: Notation,
}

impl Number {
    pub fn new(sign: Sign, notation: Notation) -> Number {
        Number {
            sign,
            notation,
        }
    }

    pub fn token(span: Span, sign: Sign, notation: Notation) -> LexToken<Number> {
        LexToken::new(Number {
            sign,
            notation,
        }, span.start, span.end)
    }

    pub fn sign(&self) -> Sign {
        self.sign
    }

    pub fn notation(&self) -> Notation {
        self.notation
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
        Ok(LexToken::new(Number::new(sign, n.get_notation()), p1, p2))
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
    buffer: String,
}

impl NumberParser {
    pub fn new() -> NumberParser {
        NumberParser {
            decimal: DecimalConfig::new(),
            hex: HexConfig::new(),
            octal: OctalConfig::new(),
            binary: BinaryConfig::new(),
            buffer: String::new(),
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

        if notation.is_some() {
            match last {
                '0' => {
                    return Ok(LexToken::new(Number::new(sign, notation.unwrap()), p1, p2));
                }
                '.' => {
                    let mut p = p2;
                    p.offset -= 1;
                    p.column -= 1;
                    r.seek(p)?;
                    return Ok(LexToken::new(Number::new(sign, notation.unwrap()), p1, p));
                }
                _ => {}
            }
        }

        let expected = match last {
            ' ' | '.' => {
                let mut expected = Vec::new();
                if sign == Sign::None {
                    if self.decimal.allow_minus {
                        expected.push(Expected::Char('-'));
                    }
                    if self.decimal.allow_plus {
                        expected.push(Expected::Char('+'));
                    }
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

    pub fn convert_number_token<N: Numerical>(&mut self, n: &LexToken<Number>, r: &mut dyn CharReader) -> Result<N, ParseErrorDetail> {
        self.convert_number(n.span(), n.term().sign(), n.term().notation(), r)
    }

    pub fn convert_number<N: Numerical>(&mut self, span: Span, sign: Sign, notation: Notation, r: &mut dyn CharReader) -> Result<N, ParseErrorDetail> {
        let res = match notation {
            Notation::Decimal => {
                let s = r.slice(span.start.offset + sign.len(), span.end.offset)?;
                parse_decimal(sign, s.as_bytes())
            }
            Notation::Hex => {
                let s = r.slice(span.start.offset + sign.len() + self.hex.prefix.len(), span.end.offset)?;
                parse_hex(sign, s.as_bytes())
            }
            Notation::Octal => {
                let s = r.slice(span.start.offset + sign.len() + self.octal.prefix.len(), span.end.offset)?;
                parse_octal(sign, s.as_bytes())
            }
            Notation::Binary => {
                let s = r.slice(span.start.offset + sign.len() + self.binary.prefix.len(), span.end.offset)?;
                parse_binary(sign, s.as_bytes())
            }
            Notation::Float | Notation::Exponent => {
                let s = r.slice(span.start.offset, span.end.offset)?;
                if self.decimal.allow_underscores {
                    self.buffer.clear();
                    for c in s.chars() {
                        if c != '_' {
                            self.buffer.push(c);
                        }
                    }
                    N::from_float_str(&self.buffer)
                } else {
                    N::from_float_str(&s)
                }
            }
        };
        res.map_err(|err| ParseErrorDetail::Numerical {
            span,
            kind: err,
        })
    }
}

impl std::fmt::Debug for NumberParser {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NumberParser")
            .field("decimal", &self.decimal)
            .field("hex", &self.hex)
            .field("octal", &self.octal)
            .field("binary", &self.binary)
            .finish()
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    Any,
    Upper,
    Lower,
}


trait NotationConfig: Sized {
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
        #[inline]
        fn is_at_prefix_or_digit<N: NotationConfig>(n: &N, r: &mut dyn CharReader) -> IoResult<bool> {
            if n.prefix().is_empty() {
                Ok(if let Some(c) = r.peek_char(0)? {
                    n.is_digit(c)
                } else {
                    false
                })
            } else {
                r.match_str(n.prefix())
            }
        }

        if self.is_enabled() {
            if let Some(c) = r.peek_char(0)? {
                if (c == '-' && self.allow_minus()) || (c == '+' && self.allow_plus()) {
                    let p = r.position();
                    r.skip_chars(1)?;
                    let res = is_at_prefix_or_digit(self, r);
                    r.seek(p)?;
                    return res;
                } else {
                    return is_at_prefix_or_digit(self, r);
                }
            }
        }
        Ok(false)
    }

    fn is_digit(&self, c: char) -> bool;

    fn get_notation(&self) -> Notation;

    fn get_expected_digit(&self) -> Expected;

    fn get_task_name(&self) -> &str {
        PARSE_TASK_NAME
    }
}


#[derive(Debug)]
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

    fn is_at_start(&self, r: &mut dyn CharReader) -> IoResult<bool> {
        if self.is_enabled() {
            if let Some(c) = r.peek_char(0)? {
                if (c == '-' && self.allow_minus()) || (c == '+' && self.allow_plus()) {
                    return Ok(true);
                } else {
                    return Ok(self.is_digit(c));
                }
            }
        }
        Ok(false)
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


#[derive(Debug)]
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


#[derive(Debug)]
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


#[derive(Debug)]
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

pub trait Numerical: Copy {
    fn from_u8(d: u8) -> Self;
    fn from_float_str(s: &str) -> Result<Self, NumericalErrorKind>;
    fn add(a: Self, b: Self) -> Option<Self>;
    fn sub(a: Self, b: Self) -> Option<Self>;
    fn mul2(a: Self) -> Option<Self>;
    fn mul8(a: Self) -> Option<Self>;
    fn mul10(a: Self) -> Option<Self>;
    fn mul16(a: Self) -> Option<Self>;
}

macro_rules! impl_numerical {
    ($ty: ty) => {
        impl Numerical for $ty {
            #[inline(always)]
            fn from_u8(d: u8) -> Self {
                d as $ty
            }

            #[inline(always)]
            fn from_float_str(s: &str) -> Result<Self, NumericalErrorKind> {
                let d: f64 = match s.parse::<f64>() {
                    Ok(d) => d,
                    Err(_) => return Err(NumericalErrorKind::Invalid),
                };
                let min = Self::min_value() as f64;
                let max = Self::max_value() as f64;
                if d < min {
                    Err(NumericalErrorKind::Underflow(d))
                } else if d > max {
                    Err(NumericalErrorKind::Overflow(d))
                } else {
                    Ok(d as $ty)
                }
            }

            #[inline(always)]
            fn add(a: Self, b: Self) -> Option<Self> {
                Self::checked_add(a, b)
            }

            #[inline(always)]
            fn sub(a: Self, b: Self) -> Option<Self> {
                Self::checked_sub(a, b)
            }

            #[inline(always)]
            fn mul2(a: Self) -> Option<Self> {
                Self::checked_mul(a, 2 as $ty)
            }

            #[inline(always)]
            fn mul8(a: Self) -> Option<Self> {
                Self::checked_mul(a, 8 as $ty)
            }

            #[inline(always)]
            fn mul10(a: Self) -> Option<Self> {
                Self::checked_mul(a, 10 as $ty)
            }

            #[inline(always)]
            fn mul16(a: Self) -> Option<Self> {
                Self::checked_mul(a, 16 as $ty)
            }
        }
    }
}

impl_numerical!(u8);
impl_numerical!(i8);
impl_numerical!(u16);
impl_numerical!(i16);
impl_numerical!(u32);
impl_numerical!(i32);
impl_numerical!(u64);
impl_numerical!(i64);
impl_numerical!(u128);
impl_numerical!(i128);
impl_numerical!(usize);
impl_numerical!(isize);

impl Numerical for f32 {
    #[inline(always)]
    fn from_u8(d: u8) -> Self {
        d as f32
    }

    #[inline(always)]
    fn from_float_str(s: &str) -> Result<Self, NumericalErrorKind> {
        s.parse::<f32>().map_err(|_| NumericalErrorKind::Invalid)
    }

    #[inline(always)]
    fn add(a: Self, b: Self) -> Option<Self> {
        Some(a + b)
    }

    #[inline(always)]
    fn sub(a: Self, b: Self) -> Option<Self> {
        Some(a - b)
    }

    #[inline(always)]
    fn mul2(a: Self) -> Option<Self> {
        Some(a * 2f32)
    }

    #[inline(always)]
    fn mul8(a: Self) -> Option<Self> {
        Some(a * 8f32)
    }

    #[inline(always)]
    fn mul10(a: Self) -> Option<Self> {
        Some(a * 10f32)
    }

    #[inline(always)]
    fn mul16(a: Self) -> Option<Self> {
        Some(a * 16f32)
    }
}

impl Numerical for f64 {
    #[inline(always)]
    fn from_u8(d: u8) -> Self {
        d as f64
    }

    #[inline(always)]
    fn from_float_str(s: &str) -> Result<Self, NumericalErrorKind> {
        s.parse::<f64>().map_err(|_| NumericalErrorKind::Invalid)
    }

    #[inline(always)]
    fn add(a: Self, b: Self) -> Option<Self> {
        Some(a + b)
    }

    #[inline(always)]
    fn sub(a: Self, b: Self) -> Option<Self> {
        Some(a - b)
    }

    #[inline(always)]
    fn mul2(a: Self) -> Option<Self> {
        Some(a * 2f64)
    }

    #[inline(always)]
    fn mul8(a: Self) -> Option<Self> {
        Some(a * 8f64)
    }

    #[inline(always)]
    fn mul10(a: Self) -> Option<Self> {
        Some(a * 10f64)
    }

    #[inline(always)]
    fn mul16(a: Self) -> Option<Self> {
        Some(a * 16f64)
    }
}

#[inline]
fn digit_dec<N: Numerical>(d: u8) -> N {
    N::from_u8(d - b'0')
}

#[inline]
fn digit_hex<N: Numerical>(d: u8) -> N {
    if d >= b'a' {
        N::from_u8(d - b'a' + 10u8)
    } else if d >= b'A' {
        N::from_u8(d - b'A' + 10u8)
    } else {
        N::from_u8(d - b'0')
    }
}

fn parse_decimal<N: Numerical>(sign: Sign, s: &[u8]) -> Result<N, NumericalErrorKind> {
    let mut n = N::from_u8(0);
    if sign != Sign::Minus {
        for &d in s {
            if d != b'_' {
                match N::mul10(n) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Overflow(std::f64::NAN)),
                }
                match N::add(n, digit_dec(d)) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Overflow(std::f64::NAN)),
                }
            }
        }
    } else {
        for &d in s {
            if d != b'_' {
                match N::mul10(n) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Underflow(std::f64::NAN)),
                }
                match N::sub(n, digit_dec(d)) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Underflow(std::f64::NAN)),
                }
            }
        }
    }
    Ok(n)
}

fn parse_octal<N: Numerical>(sign: Sign, s: &[u8]) -> Result<N, NumericalErrorKind> {
    let mut n = N::from_u8(0);
    if sign != Sign::Minus {
        for &d in s {
            if d != b'_' {
                match N::mul8(n) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Overflow(std::f64::NAN)),
                }
                match N::add(n, digit_dec(d)) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Overflow(std::f64::NAN)),
                }
            }
        }
    } else {
        for &d in s {
            if d != b'_' {
                match N::mul8(n) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Underflow(std::f64::NAN)),
                }
                match N::sub(n, digit_dec(d)) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Underflow(std::f64::NAN)),
                }
            }
        }
    }
    Ok(n)
}

fn parse_binary<N: Numerical>(sign: Sign, s: &[u8]) -> Result<N, NumericalErrorKind> {
    let mut n = N::from_u8(0);
    if sign != Sign::Minus {
        for &d in s {
            if d != b'_' {
                match N::mul2(n) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Overflow(std::f64::NAN)),
                }
                match N::add(n, digit_dec(d)) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Overflow(std::f64::NAN)),
                }
            }
        }
    } else {
        for &d in s {
            if d != b'_' {
                match N::mul2(n) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Underflow(std::f64::NAN)),
                }
                match N::sub(n, digit_dec(d)) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Underflow(std::f64::NAN)),
                }
            }
        }
    }
    Ok(n)
}

fn parse_hex<N: Numerical>(sign: Sign, s: &[u8]) -> Result<N, NumericalErrorKind> {
    let mut n = N::from_u8(0);
    if sign != Sign::Minus {
        for &d in s {
            if d != b'_' {
                match N::mul16(n) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Overflow(std::f64::NAN)),
                }
                match N::add(n, digit_hex(d)) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Overflow(std::f64::NAN)),
                }
            }
        }
    } else {
        for &d in s {
            if d != b'_' {
                match N::mul16(n) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Underflow(std::f64::NAN)),
                }
                match N::sub(n, digit_hex(d)) {
                    Some(a) => n = a,
                    None => return Err(NumericalErrorKind::Underflow(std::f64::NAN)),
                }
            }
        }
    }
    Ok(n)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_exponent() {
        let mut np = NumberParser::new();
        let mut r = MemCharReader::new(b"-123456.8e-3");
        let n = np.parse_number(&mut r).unwrap();
        assert_eq!(n.term().sign(), Sign::Minus);
        assert_eq!(n.term().notation(), Notation::Exponent);
        assert_eq!(np.convert_number_token::<i32>(&n, &mut r).unwrap(), -123);
        assert_eq!(np.convert_number_token::<f32>(&n, &mut r).unwrap(), -123.4568f32);
        assert_eq!(np.convert_number_token::<f64>(&n, &mut r).unwrap(), -123.4568f64);
    }

    #[test]
    fn can_parse_decimal() {
        let mut np = NumberParser::new();
        let mut r = MemCharReader::new(b"-123456");
        let n = np.parse_number(&mut r).unwrap();
        assert_eq!(n.term().sign(), Sign::Minus);
        assert_eq!(n.term().notation(), Notation::Decimal);
        assert_eq!(np.convert_number_token::<i32>(&n, &mut r).unwrap(), -123456);
        assert_eq!(np.convert_number_token::<f32>(&n, &mut r).unwrap(), -123456f32);
        assert_eq!(np.convert_number_token::<f64>(&n, &mut r).unwrap(), -123456f64);
    }

    #[test]
    fn can_parse_decimal_ending_with_dot() {
        let mut np = NumberParser::new();
        let mut r = MemCharReader::new(b"123456..");
        let n = np.parse_number(&mut r).unwrap();
        assert_eq!(n.term().sign(), Sign::None);
        assert_eq!(n.term().notation(), Notation::Decimal);
        assert_eq!(n.start().offset, 0);
        assert_eq!(n.start().column, 0);
        assert_eq!(n.end().offset, 6);
        assert_eq!(n.end().column, 6);
        assert_eq!(np.convert_number_token::<i32>(&n, &mut r).unwrap(), 123456);
    }

    #[test]
    fn can_parse_float() {
        let mut np = NumberParser::new();
        let mut r = MemCharReader::new(b"123.456");
        let n = np.parse_number(&mut r).unwrap();
        assert_eq!(n.term().sign(), Sign::None);
        assert_eq!(n.term().notation(), Notation::Float);
        assert_eq!(np.convert_number_token::<i32>(&n, &mut r).unwrap(), 123);
        assert_eq!(np.convert_number_token::<f32>(&n, &mut r).unwrap(), 123.456f32);
        assert_eq!(np.convert_number_token::<f64>(&n, &mut r).unwrap(), 123.456f64);
    }

    #[test]
    fn can_parse_hex() {
        let mut np = NumberParser::new();
        let mut r = MemCharReader::new(b"0xaaff");
        let n = np.parse_number(&mut r).unwrap();
        assert_eq!(n.term().sign(), Sign::None);
        assert_eq!(n.term().notation(), Notation::Hex);
        assert_eq!(np.convert_number_token::<i32>(&n, &mut r).unwrap(), 0xAAFF);
        assert_eq!(np.convert_number_token::<f32>(&n, &mut r).unwrap(), 0xAAFF as f32);
        assert_eq!(np.convert_number_token::<f64>(&n, &mut r).unwrap(), 0xAAFF as f64);
    }

    #[test]
    fn can_parse_octal() {
        let mut np = NumberParser::new();
        let mut r = MemCharReader::new(b"0o777");
        let n = np.parse_number(&mut r).unwrap();
        assert_eq!(n.term().sign(), Sign::None);
        assert_eq!(n.term().notation(), Notation::Octal);
        assert_eq!(np.convert_number_token::<i32>(&n, &mut r).unwrap(), 0o777);
        assert_eq!(np.convert_number_token::<f32>(&n, &mut r).unwrap(), 0o777 as f32);
        assert_eq!(np.convert_number_token::<f64>(&n, &mut r).unwrap(), 0o777 as f64);
    }

    #[test]
    fn can_parse_binary() {
        let mut np = NumberParser::new();
        let mut r = MemCharReader::new(b"0b10010011");
        let n = np.parse_number(&mut r).unwrap();
        assert_eq!(n.term().sign(), Sign::None);
        assert_eq!(n.term().notation(), Notation::Binary);
        assert_eq!(np.convert_number_token::<i32>(&n, &mut r).unwrap(), 0b10010011);
        assert_eq!(np.convert_number_token::<f32>(&n, &mut r).unwrap(), 0b10010011 as f32);
        assert_eq!(np.convert_number_token::<f64>(&n, &mut r).unwrap(), 0b10010011 as f64);
    }
}