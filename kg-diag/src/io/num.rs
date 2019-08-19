use super::*;

use std::num::NonZeroUsize;

use regex::Regex;

pub enum Number {
    Int(i128),
    Float(f64),
}

pub struct NumberParser {
    re: Regex,
    decimal: Option<NonZeroUsize>,
    hex: Option<NonZeroUsize>,
    octal: Option<NonZeroUsize>,
    binary: Option<NonZeroUsize>,
    float: Option<NonZeroUsize>,
}

impl NumberParser {

}


pub struct NumberParserBuilder<'a> {
    decimal: DecimalNotationBuilder<'a>,

}

impl<'a> NumberParserBuilder<'a> {
    pub fn new() -> NumberParserBuilder<'a> {
        NumberParserBuilder {
            decimal: DecimalNotationBuilder::new(),
        }
    }

    pub fn decimal(&mut self) -> &mut DecimalNotationBuilder<'a> {
        &mut self.decimal
    }

    pub fn build(mut self) -> String {
        let mut s = String::new();
        let dec = self.decimal.into_config();
        if dec.enabled {
            if !s.is_empty() {
                s.push('|')
            }
            s.push_str("(^");
            if dec.allow_plus || dec.allow_minus {
                s.push('[');
                if dec.allow_minus {
                    s.push('-');
                }
                if dec.allow_plus {
                    s.push('+');
                }
                s.push_str("]?");
            }
            s.push(')');
        }
        s
    }
}

struct NotationConfig<'a> {
    enabled: bool,
    prefix: &'a str,
    digit: &'a str,
    allow_minus: bool,
    allow_plus: bool,
    allow_underscores: bool,
    case_sensitive: bool,
}

pub struct DecimalNotationBuilder<'a>(NotationConfig<'a>);

impl<'a> DecimalNotationBuilder<'a> {
    fn new() -> DecimalNotationBuilder<'a> {
        DecimalNotationBuilder(NotationConfig {
            enabled: false,
            prefix: "",
            digit: "[0-9]",
            allow_minus: false,
            allow_plus: false,
            allow_underscores: false,
            case_sensitive: false,
        })
    }

    pub fn enabled(&mut self, enabled: bool) -> &mut DecimalNotationBuilder<'a> {
        self.0.enabled = enabled;
        self
    }

    pub fn allow_minus(&mut self, allow_minus: bool) -> &mut DecimalNotationBuilder<'a> {
        self.0.allow_minus = allow_minus;
        self
    }

    pub fn allow_plus(&mut self, allow_plus: bool) -> &mut DecimalNotationBuilder<'a> {
        self.0.allow_plus = allow_plus;
        self
    }

    pub fn allow_underscore(&mut self, allow_underscores: bool) -> &mut DecimalNotationBuilder<'a> {
        self.0.allow_underscores = allow_underscores;
        self
    }

    fn into_config(self) -> NotationConfig<'a> {
        self.0
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_decimal() {
        let mut b = NumberParserBuilder::new();
        b.decimal().enabled(true).allow_minus(true).allow_plus(false).allow_underscore(false);
        let re = b.build();

        println!("{:?}", re);

    }
}