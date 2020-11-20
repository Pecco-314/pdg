use crate::token::*;
use num::cast::ToPrimitive;
use simple_combinators::{
    combinator::{many1, optional},
    parser::*,
    ParseError, Parser,
};
use std::ops::Range;
use Parameter::*;
use RandomString::*;
use Token::*;

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

fn parameter() -> impl Parser<ParseResult = Parameter> {
    number()
        .map(|i| Parameter::Int(i))
        .or(many1(alpha()).map(|s| Parameter::Enum(s)))
        .or(any()
            .between(char('\''), char('\''))
            .map(|c| Parameter::Char(c)))
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
        _ => Err(ParseError),
    })
}

fn array_token() -> impl Parser<ParseResult = Token> {
    char('A')
        .with(parameters())
        .flat_map(|v| match &v[..] {
            [Int(a)] => Ok(a.to_usize().ok_or(ParseError)?),
            _ => Err(ParseError),
        })
        .and(repeated())
        .map(|(times, v)| Gen(Array(times, v)))
}

fn testcase_token() -> impl Parser<ParseResult = Token> {
    char('T')
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
                .or(array_token())
                .or(testcase_token())
                .or(random_string_token())
                .or(cmp_op()),
        )
        .skip(spaces())
}
