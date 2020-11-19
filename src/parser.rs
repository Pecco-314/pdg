use crate::token::*;
use simple_combinators::{
    combinator::optional,
    parser::{char, into_num, size, spaces, string},
    ParseError, Parser,
};
use std::ops::Range;

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

fn array_token() -> impl Parser<ParseResult = Token> {
    char('A')
        .with(into_num())
        .and(repeated())
        .map(|(times, v)| Token::Gen(Array(times, v)))
}

fn testcase_token() -> impl Parser<ParseResult = Token> {
    char('T')
        .with(into_num())
        .and(repeated())
        .map(|(times, v)| Token::Gen(TestCase(times, v)))
}

fn random_integer_token() -> impl Parser<ParseResult = Token> {
    char('i')
        .with(into_num())
        .and(optional(char(',').with(into_num())))
        .map(|(a, opt)| match opt {
            Some(b) => Token::Gen(RandomIntegerBetween(a, b)),
            None => Token::Gen(RandomIntegerNoGreaterThan(a)),
        })
}

pub fn file_range() -> impl Parser<ParseResult = Range<usize>> {
    spaces()
        .skip(string(":>"))
        .skip(spaces())
        .with(size())
        .skip(spaces())
        .and(optional(string("..").skip(spaces()).with(size())))
        .map(|(a, op)| match op {
            Some(b) => a..b + 1,
            None => a..a + 1,
        })
}

pub fn token() -> impl Parser<ParseResult = Token> {
    spaces()
        .with(
            random_integer_token()
                .or(into_num().map(|i| Token::Gen(ConstantInteger(i))))
                .or(array_token())
                .or(testcase_token())
                .or(char('/').map(|_| Token::Gen(NewLine)))
                .or(char('<').map(|_| Token::Op(LessThan)))
                .or(char('>').map(|_| Token::Op(GreaterThan)))
                .or(string("<=").map(|_| Token::Op(NoGreaterThan)))
                .or(string(">=").map(|_| Token::Op(NoLessThan))),
        )
        .skip(spaces())
}
