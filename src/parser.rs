use crate::token::*;
use num::cast::ToPrimitive;
use simple_combinators::{combinator::optional, parser::*, ParseError, Parser};
use std::ops::Range;
use ConfigItem::*;
use Parameter::*;
use RandomString::*;
use Token::*;

pub fn config_item() -> impl Parser<ParseResult = ConfigItem> {
    spaces()
        .with(
            string("#fold")
                .with(parameters())
                .flat_map(|v| match &v[..] {
                    [Str(s)] => Ok(Fold(s.clone())),
                    _ => Err(ParseError),
                })
                .or(string("#pause")
                    .with(parameters())
                    .flat_map(|v| match &v[..] {
                        [Bool(b)] => Ok(Pause(*b)),
                        _ => Err(ParseError),
                    }))
                .or(string("#prefix")
                    .with(parameters())
                    .flat_map(|v| match &v[..] {
                        [Str(s)] => Ok(Prefix(s.clone())),
                        _ => Err(ParseError),
                    })),
        )
        .skip(spaces())
}
#[derive(Copy, Clone, Debug)]
struct ConfigParser;
impl Parser for ConfigParser {
    type ParseResult = Config;
    fn parse(&self, buf: &mut &str) -> Result<Self::ParseResult, ParseError> {
        let mut config = Config {
            ..Default::default()
        };
        for item in config_item().iter(buf) {
            match item {
                Fold(s) => {
                    config.fold = Some(s);
                }
                Pause(b) => {
                    config.pause = Some(b);
                }
                Prefix(b) => {
                    config.prefix = Some(b);
                }
            }
        }
        Ok(config)
    }
}
pub fn config() -> impl Parser<ParseResult = Config> {
    ConfigParser
}

pub fn file_range() -> impl Parser<ParseResult = Range<usize>> {
    spaces()
        .skip(string(":>"))
        .skip(spaces())
        .with(number())
        .skip(spaces())
        .and(optional(string("..").skip(spaces()).with(number())))
        .map(|(a, op): (usize, Option<usize>)| match op {
            Some(b) => a..b + 1,
            None => a..a + 1,
        })
}

fn int_parameter() -> impl Parser<ParseResult = Parameter> {
    number()
        .map(|i| Int(i))
        .or(
            char('!').with(random_integer_token().flat_map(|token| match token {
                Gen(gen) => Ok(gen.generate().ok_or(ParseError)?),
                _ => Err(ParseError),
            })),
        )
}

fn str_parameter() -> impl Parser<ParseResult = Parameter> {
    quoted_string()
        .map(|s| Str(s))
        .or(
            char('!').with(random_string_token().flat_map(|token| match token {
                Gen(gen) => Ok(gen.generate().ok_or(ParseError)?),
                _ => Err(ParseError),
            })),
        )
}

#[derive(Copy, Clone)]
struct ParameterParser;
impl Parser for ParameterParser {
    type ParseResult = Parameter;
    fn parse(&self, buf: &mut &str) -> Result<Self::ParseResult, ParseError> {
        int_parameter()
            .or(str_parameter())
            .or(any().between(char('\''), char('\'')).map(|c| Char(c)))
            .or(string("true").map(|_| Bool(true)))
            .or(string("false").map(|_| Bool(false)))
            .or(word().map(|e| Enum(e)))
            .parse(buf)
    }
}
fn parameter() -> impl Parser<ParseResult = Parameter> {
    ParameterParser
}

fn parameters() -> impl Parser<ParseResult = Vec<Parameter>> {
    parameter()
        .sep_by(spaces().with(char(',').skip(spaces())))
        .between(char('[').skip(spaces()), spaces().with(char(']')))
        .or(parameter().sep_by(spaces().with(char(',')).skip(spaces())))
}

#[derive(Copy, Clone)]
struct Repeated;
impl Parser for Repeated {
    type ParseResult = Vec<Token>;
    fn parse(&self, buf: &mut &str) -> Result<Self::ParseResult, ParseError> {
        let res = spaces().skip(char('{')).parse(buf);
        let mut v = Vec::new();
        match res {
            Ok(_) => {
                for i in token().iter(buf) {
                    v.push(i.clone());
                }
                spaces().skip(char('}')).parse(buf)?;
            }
            Err(_) => {
                v.push(token().iter(buf).next().ok_or(ParseError)?.clone());
            }
        };
        Ok(v)
    }
}
fn repeated() -> impl Parser<ParseResult = Vec<Token>> {
    Repeated
}

fn random_integer_token() -> impl Parser<ParseResult = Token> {
    char('i').with(parameters()).flat_map(|v| match &v[..] {
        [Int(a)] => Ok(Gen(RandomIntegerNoGreaterThan(*a))),
        [Int(a), Int(b)] => Ok(Gen(RandomIntegerBetween(*a, *b))),
        _ => Err(ParseError),
    })
}

fn random_string_token() -> impl Parser<ParseResult = Token> {
    char('s').with(parameters()).flat_map(|v| match &v[..] {
        [Int(a)] => Ok(Gen(RandomString(Lower(a.to_usize().ok_or(ParseError)?)))),
        [Enum(e), Int(a)] if e == "lower" => {
            Ok(Gen(RandomString(Lower(a.to_usize().ok_or(ParseError)?))))
        }
        [Enum(e), Int(a)] if e == "upper" => {
            Ok(Gen(RandomString(Upper(a.to_usize().ok_or(ParseError)?))))
        }
        [Enum(e), Int(a)] if e == "alpha" => {
            Ok(Gen(RandomString(Alpha(a.to_usize().ok_or(ParseError)?))))
        }
        [Enum(e), Int(a)] if e == "bin" => {
            Ok(Gen(RandomString(Bin(a.to_usize().ok_or(ParseError)?))))
        }
        [Enum(e), Int(a)] if e == "oct" => {
            Ok(Gen(RandomString(Oct(a.to_usize().ok_or(ParseError)?))))
        }
        [Enum(e), Int(a)] if e == "dec" => {
            Ok(Gen(RandomString(Dec(a.to_usize().ok_or(ParseError)?))))
        }
        [Enum(e), Int(a)] if e == "hexlower" => {
            Ok(Gen(RandomString(HexLower(a.to_usize().ok_or(ParseError)?))))
        }
        [Enum(e), Int(a)] if e == "hexupper" => {
            Ok(Gen(RandomString(HexUpper(a.to_usize().ok_or(ParseError)?))))
        }
        [Enum(e), Int(a)] if e == "alnum" => {
            Ok(Gen(RandomString(Alnum(a.to_usize().ok_or(ParseError)?))))
        }
        [Enum(e), Int(a)] if e == "graph" => {
            Ok(Gen(RandomString(Graph(a.to_usize().ok_or(ParseError)?))))
        }
        [Enum(e), Str(s), Int(a)] if e == "oneof" => Ok(Gen(RandomString(OneOf(
            s.clone(),
            a.to_usize().ok_or(ParseError)?,
        )))),
        [Enum(e), Char(l), Char(r), Int(a)] if e == "between" => Ok(Gen(RandomString(Between(
            *l,
            *r,
            a.to_usize().ok_or(ParseError)?,
        )))),
        _ => Err(ParseError),
    })
}

fn repeated_token() -> impl Parser<ParseResult = Token> {
    char('X')
        .with(parameters())
        .flat_map(|v| match &v[..] {
            [Int(a)] => Ok(a.to_usize().ok_or(ParseError)?),
            _ => Err(ParseError),
        })
        .and(repeated())
        .map(|(times, v)| Gen(Array(times, v)))
}

fn array_token() -> impl Parser<ParseResult = Token> {
    char('A')
        .with(parameters())
        .flat_map(|v| match &v[..] {
            [Int(a)] => Ok(a.to_usize().ok_or(ParseError)?),
            _ => Err(ParseError),
        })
        .and(repeated())
        .map(|(times, v)| Gen(TestCase(times, v)))
}

pub fn cmp_op() -> impl Parser<ParseResult = Token> {
    char('<')
        .map(|_| Op(LessThan))
        .or(char('>').map(|_| Op(GreaterThan)))
        .or(string("<=").map(|_| Op(NoGreaterThan)))
        .or(string(">=").map(|_| Op(NoLessThan)))
}

pub fn constant() -> impl Parser<ParseResult = Token> {
    quoted_string()
        .map(|s| Gen(ConstantString(s)))
        .or(number().map(|i| Gen(ConstantInteger(i))))
        .or(char('/').map(|_| Gen(NewLine)))
}

pub fn token() -> impl Parser<ParseResult = Token> {
    spaces()
        .with(
            constant()
                .or(random_integer_token())
                .or(repeated_token())
                .or(array_token())
                .or(random_string_token())
                .or(cmp_op()),
        )
        .skip(spaces())
}
