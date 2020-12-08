use crate::{
    details::warning_info,
    token::{Parameter::*, RandomString::*, Token::*, *},
};
use simple_combinators::{
    combinator::{attempt, many1, optional, preview, satisfy},
    parser::*,
    ParseError, Parser,
};
use std::{collections::HashMap, ops::Range};

static REGISTER: &[&str] = &["prefix", "pause", "folder", "std"];
#[derive(Copy, Clone)]
struct ConfigParser;
impl Parser for ConfigParser {
    type ParseResult = Config;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let mut map = HashMap::new();
        let config_item = spaces()
            .skip(char('#'))
            .with(word())
            .and(optional(parameters()));
        for (k, op) in config_item.iter(buf) {
            if !REGISTER.contains(&k.as_str()) {
                warning_info(&format!("unsupported config: {}", k));
            }
            match op {
                Some(v) => {
                    map.insert(k, v);
                }
                None => warning_info(&format!(
                    "Config '{}' does not have parameters with correct formats",
                    k
                )),
            }
        }
        Ok(map)
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
                .or(attempt(integer_pair_token()))
                .or(random_integer_token())
                .or(random_string_token())
                .or(repeated_token())
                .or(array_token())
                .or(distribute_token())
                .or(token_group()),
        )
        .skip(spaces())
}

pub fn integer_pair_token() -> impl Parser<ParseResult = Token> {
    random_integer()
        .and(
            spaces()
                .with(
                    string("<=")
                        .or(string(">="))
                        .or(string("<"))
                        .or(string(">")),
                )
                .skip(spaces()),
        )
        .and(random_integer())
        .flat_map(|tuple| match tuple {
            ((t1, op), t2) => Some(RandomIntegerPair(
                t1.left(),
                t1.right(),
                t2.left(),
                t2.right(),
                match_op(op)?,
            )),
        })
}

fn match_op(op: &str) -> Option<Cmp> {
    match op {
        "<" => Some(Cmp::LessThan),
        ">" => Some(Cmp::GreaterThan),
        "<=" => Some(Cmp::NoGreaterThan),
        ">=" => Some(Cmp::NoLessThan),
        _ => None,
    }
}

fn normal_parameter() -> impl Parser<ParseResult = Parameter> {
    attempt(quoted_string())
        .map(|s| Str(StrParameter::Confirm(s)))
        .or(number().map(|i| Int(IntParameter::Confirm(i))))
}

fn exclmark_parameter() -> impl Parser<ParseResult = Parameter> {
    char('!').with(
        random_string_token()
            .flat_map(|token| token.generate())
            .or(attempt(
                random_integer_token().flat_map(|token| token.generate()),
            )),
    )
}

fn quesmark_parameter() -> impl Parser<ParseResult = Parameter> {
    char('?').with(
        random_string_token()
            .map(|token| (Str(StrParameter::Lazy(Box::new(token)))))
            .or(random_integer_token().map(|token| Int(IntParameter::Lazy(Box::new(token))))),
    )
}

fn random_integer() -> impl Parser<ParseResult = crate::token::RandomInteger> {
    use crate::token::RandomInteger::*;
    char('i')
        .with(attempt(parameters()))
        .flat_map(|v| match &v[..] {
            [Int(a)] => Some(NoGreaterThan(a.clone())),
            [Int(a), Int(b)] => Some(Between(a.clone(), b.clone())),
            _ => None,
        })
}

fn random_integer_token() -> impl Parser<ParseResult = Token> {
    random_integer().map(|r| Token::RandomInteger(r))
}

fn random_string_token() -> impl Parser<ParseResult = Token> {
    char('s')
        .with(attempt(parameters()))
        .flat_map(|v| match &v[..] {
            [Int(t)] => Some(RandomString(Lower(t.clone()))),
            [Enum(e), Int(t)] if e == "lower" => Some(RandomString(Lower(t.clone()))),
            [Enum(e), Int(t)] if e == "upper" => Some(RandomString(Upper(t.clone()))),
            [Enum(e), Int(t)] if e == "alpha" => Some(RandomString(Alpha(t.clone()))),
            [Enum(e), Int(t)] if e == "bin" => Some(RandomString(Bin(t.clone()))),
            [Enum(e), Int(t)] if e == "oct" => Some(RandomString(Oct(t.clone()))),
            [Enum(e), Int(t)] if e == "dec" => Some(RandomString(Dec(t.clone()))),
            [Enum(e), Int(t)] if e == "hexlower" => Some(RandomString(HexLower(t.clone()))),
            [Enum(e), Int(t)] if e == "hexupper" => Some(RandomString(HexUpper(t.clone()))),
            [Enum(e), Int(t)] if e == "alnum" => Some(RandomString(Alnum(t.clone()))),
            [Enum(e), Int(t)] if e == "graph" => Some(RandomString(Graph(t.clone()))),
            [Enum(e), Str(s), Int(t)] if e == "oneof" => {
                Some(RandomString(OneOf(s.clone(), t.clone())))
            }
            [Enum(e), Char(l), Char(r), Int(t)] if e == "between" => {
                Some(RandomString(Between(*l, *r, t.clone())))
            }
            _ => None,
        })
}

pub fn constant() -> impl Parser<ParseResult = Token> {
    quoted_string()
        .map(|s| ConstantString(s))
        .or(number().map(|i| ConstantInteger(i)))
        .or(char('/').map(|_| NewLine))
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
            .map(|v| TokenGroup(v))
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
        char('D')
            .with(spaces())
            .with(
                parameter()
                    .skip(spaces())
                    .skip(char(':'))
                    .and(token())
                    .flat_map(|(p, t)| match p {
                        Int(i) => Some((i, t)),
                        _ => None,
                    })
                    .sep_by(spaces().skip(char(';')).skip(spaces()))
                    .between(char('{').skip(spaces()), spaces().with(char('}')))
                    .map(|v| Distribute(v)),
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
        char('X')
            .with(parameters())
            .flat_map(|v| match &v[..] {
                [Int(ip)] => Some(ip.clone()),
                _ => None,
            })
            .and(token())
            .map(|(ip, token)| Repeat(ip, Box::new(token)))
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
        char('A')
            .with(parameters())
            .flat_map(|v| match &v[..] {
                [Int(ip)] => Some(ip.clone()),
                _ => None,
            })
            .and(token())
            .map(|(ip, token)| Array(ip, Box::new(token)))
            .parse(buf)
    }
}
fn array_token() -> impl Parser<ParseResult = Token> {
    ArrayTokenParser
}
