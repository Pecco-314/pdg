use crate::{
    resolve,
    token::{ConfigItem::*, Gen::*, Parameter::*, RandomString::*, Token::*, *},
};
use num::cast::ToPrimitive;
use simple_combinators::{
    combinator::{attempt, many1, optional, preview, satisfy},
    parser::*,
    ParseError, Parser,
};
use std::ops::Range;

pub fn config_item() -> impl Parser<ParseResult = ConfigItem> {
    use StrParameter::*;
    spaces()
        .with(
            string("#folder")
                .with(parameters())
                .flat_map(|v| match &v[..] {
                    [Str(s)] => Some(Folder(resolve!(s, str))),
                    _ => None,
                })
                .or(string("#pause")
                    .with(parameters())
                    .flat_map(|v| match &v[..] {
                        [Bool(b)] => Some(Pause(*b)),
                        _ => None,
                    }))
                .or(string("#std")
                    .with(parameters())
                    .flat_map(|v| match &v[..] {
                        [Str(s)] => Some(Std(resolve!(s, str))),
                        _ => None,
                    }))
                .or(string("#prefix")
                    .with(parameters())
                    .flat_map(|v| match &v[..] {
                        [Str(s)] => Some(Prefix(resolve!(s, str))),
                        _ => None,
                    })),
        )
        .skip(spaces())
}
#[derive(Copy, Clone, Debug)]
struct ConfigParser;
impl Parser for ConfigParser {
    type ParseResult = Config;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let mut config = Config {
            ..Default::default()
        };
        for item in config_item().iter(buf) {
            match item {
                Folder(s) => {
                    config.folder = Some(s);
                }
                Pause(b) => {
                    config.pause = Some(b);
                }
                Prefix(b) => {
                    config.prefix = Some(b);
                }
                Std(b) => {
                    config.std = Some(b);
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

pub fn token() -> impl Parser<ParseResult = Token> {
    spaces()
        .with(
            attempt(constant())
                .or(random_integer_token())
                .or(random_string_token())
                .or(repeated_token())
                .or(array_token())
                .or(distribute_token())
                .or(token_group())
                .or(cmp_op()),
        )
        .skip(spaces())
}

fn normal_parameter() -> impl Parser<ParseResult = Parameter> {
    attempt(quoted_string())
        .map(|s| Str(StrParameter::Confirm(s)))
        .or(number().map(|i| Int(IntParameter::Confirm(i))))
}

fn exclmark_parameter() -> impl Parser<ParseResult = Parameter> {
    char('!').with(
        random_string_token()
            .flat_map(|token| match token {
                Gen(gen) => Some(gen.generate()?),
                _ => None,
            })
            .or(attempt(random_integer_token().flat_map(
                |token| match token {
                    Gen(gen) => Some(gen.generate()?),
                    _ => None,
                },
            ))),
    )
}

fn quesmark_parameter() -> impl Parser<ParseResult = Parameter> {
    char('?').with(
        random_string_token()
            .flat_map(|token| match token {
                Gen(gen) => Some(Str(StrParameter::Lazy(Box::new(gen)))),
                _ => None,
            })
            .or(attempt(random_integer_token().flat_map(
                |token| match token {
                    Gen(gen) => Some(Int(IntParameter::Lazy(Box::new(gen)))),
                    _ => None,
                },
            ))),
    )
}

fn random_integer_token() -> impl Parser<ParseResult = Token> {
    use crate::token::RandomInteger::*;
    char('i')
        .with(attempt(parameters()))
        .flat_map(|v| match &v[..] {
            [Int(a)] => Some(Gen(RandomInteger(NoGreaterThan(a.clone())))),
            [Int(a), Int(b)] => Some(Gen(RandomInteger(Between(a.clone(), b.clone())))),
            _ => None,
        })
}

fn random_string_token() -> impl Parser<ParseResult = Token> {
    char('s')
        .with(attempt(parameters()))
        .flat_map(|v| match &v[..] {
            [Int(t)] => Some(Gen(RandomString(Lower(t.clone())))),
            [Enum(e), Int(t)] if e == "lower" => Some(Gen(RandomString(Lower(t.clone())))),
            [Enum(e), Int(t)] if e == "upper" => Some(Gen(RandomString(Upper(t.clone())))),
            [Enum(e), Int(t)] if e == "alpha" => Some(Gen(RandomString(Alpha(t.clone())))),
            [Enum(e), Int(t)] if e == "bin" => Some(Gen(RandomString(Bin(t.clone())))),
            [Enum(e), Int(t)] if e == "oct" => Some(Gen(RandomString(Oct(t.clone())))),
            [Enum(e), Int(t)] if e == "dec" => Some(Gen(RandomString(Dec(t.clone())))),
            [Enum(e), Int(t)] if e == "hexlower" => Some(Gen(RandomString(HexLower(t.clone())))),
            [Enum(e), Int(t)] if e == "hexupper" => Some(Gen(RandomString(HexUpper(t.clone())))),
            [Enum(e), Int(t)] if e == "alnum" => Some(Gen(RandomString(Alnum(t.clone())))),
            [Enum(e), Int(t)] if e == "graph" => Some(Gen(RandomString(Graph(t.clone())))),
            [Enum(e), Str(s), Int(t)] if e == "oneof" => {
                Some(Gen(RandomString(OneOf(s.clone(), t.clone()))))
            }
            [Enum(e), Char(l), Char(r), Int(t)] if e == "between" => {
                Some(Gen(RandomString(Between(*l, *r, t.clone()))))
            }
            _ => None,
        })
}

pub fn cmp_op() -> impl Parser<ParseResult = Token> {
    use crate::token::Op::*;
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

#[derive(Copy, Clone)]
struct ParameterParser;
impl Parser for ParameterParser {
    type ParseResult = Parameter;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        attempt(normal_parameter())
            .or(exclmark_parameter())
            .or(quesmark_parameter())
            .or(attempt(
                any().between(char('\''), char('\'')).map(|c| Char(c)),
            ))
            .or(attempt(
                string("true")
                    .skip(preview(satisfy(|c: char| !c.is_alphabetic())))
                    .map(|_| Bool(true)),
            ))
            .or(attempt(
                string("false")
                    .skip(preview(satisfy(|c: char| !c.is_alphabetic())))
                    .map(|_| Bool(false)),
            ))
            .or(word().map(|e| Enum(e)))
            .parse(buf)
    }
}
fn parameter() -> ParameterParser {
    ParameterParser
}

fn parameters() -> impl Parser<ParseResult = Vec<Parameter>> {
    parameter()
        .sep_by(spaces().with(char(',')).skip(spaces()))
        .between(char('[').skip(spaces()), spaces().with(char(']')))
        .or(parameter().sep_by(char(',')))
}

#[derive(Copy, Clone)]
struct TokenGroupParser;
impl Parser for TokenGroupParser {
    type ParseResult = Token;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        many1(token())
            .between(char('{').skip(spaces()), spaces().with(char('}')))
            .map(|v| Gen(TokenGroup(v)))
            .parse(buf)
    }
}
fn token_group() -> impl Parser<ParseResult = Token> {
    TokenGroupParser
}

#[derive(Copy, Clone)]
struct DistributeToken;
impl Parser for DistributeToken {
    type ParseResult = Token;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        char('D').parse(buf)?;
        spaces()
            .with(
                parameter()
                    .skip(spaces())
                    .skip(char(':'))
                    .and(token())
                    .flat_map(|(p, t)| match p {
                        Int(i) => Some((i, t)),
                        _ => None,
                    })
                    .sep_by(spaces().skip(char(',')).skip(spaces()))
                    .between(char('{').skip(spaces()), spaces().with(char('}')))
                    .map(|v| Gen(Distribute(v))),
            )
            .parse(buf)
    }
}
fn distribute_token() -> impl Parser<ParseResult = Token> {
    DistributeToken
}

#[derive(Copy, Clone)]
struct RepeatedTokenParser;
impl Parser for RepeatedTokenParser {
    type ParseResult = Token;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        char('X').parse(buf)?;
        parameters()
            .flat_map(|v| match &v[..] {
                [Int(IntParameter::Confirm(a))] => Some(a.to_usize()?),
                _ => None,
            })
            .and(token())
            .map(|(times, token)| Gen(Repeat(times, Box::new(token))))
            .parse(buf)
    }
}
fn repeated_token() -> impl Parser<ParseResult = Token> {
    RepeatedTokenParser
}

#[derive(Copy, Clone)]
struct ArrayTokenParser;
impl Parser for ArrayTokenParser {
    type ParseResult = Token;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        char('A').parse(buf)?;
        parameters()
            .flat_map(|v| match &v[..] {
                [Int(IntParameter::Confirm(a))] => Some(a.to_usize()?),
                _ => None,
            })
            .and(token())
            .map(|(times, token)| Gen(Array(times, Box::new(token))))
            .parse(buf)
    }
}
fn array_token() -> impl Parser<ParseResult = Token> {
    ArrayTokenParser
}
