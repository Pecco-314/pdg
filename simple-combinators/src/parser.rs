use super::combinator::*;
use super::{ParseError, Parser};

/// 匹配指定字符
pub fn char(expected: char) -> impl Parser<ParseResult = char> {
    satisfy(move |c| c == expected)
}

/// 匹配任意字符
pub fn any() -> impl Parser<ParseResult = char> {
    satisfy(|_| true)
}

/// 匹配十进制数字，返回字符
pub fn digit() -> impl Parser<ParseResult = char> {
    satisfy(|c: char| c.is_digit(10))
}

/// 匹配拉丁字母
pub fn letter() -> impl Parser<ParseResult = char> {
    satisfy(|c: char| c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z')
}

/// 匹配空白符
pub fn space() -> impl Parser<ParseResult = char> {
    satisfy(|c: char| c.is_whitespace())
}

/// 匹配并跳过任意数目的空白符
pub fn spaces() -> impl Parser<ParseResult = ()> {
    many(ignore(space()))
}

/// 匹配存在于某一字符串内的字符
pub fn one_of(s: &str) -> impl Parser<ParseResult = char> + '_ {
    satisfy(move |c| s.contains(c))
}

pub fn number() -> impl Parser<ParseResult = f64> {
    many1(one_of("0123456789Ee-.")).flat_map(|s: String| s.parse::<f64>())
}

pub fn into_integer() -> impl Parser<ParseResult = i64> {
    number().flat_map(|x| (x as i64 as f64 == x).then_some(x as i64).ok_or(ParseError))
}

pub fn integer() -> impl Parser<ParseResult = i64> {
    many1(one_of("-0123456789")).flat_map(|s: String| s.parse::<i64>())
}

#[derive(Copy, Clone)]
pub struct Str<'a> {
    string: &'a str,
}
impl<'a> Parser for Str<'a> {
    type ParseResult = &'a str;
    fn parse(&self, buf: &mut &str) -> Result<Self::ParseResult, ParseError> {
        (&buf[..self.string.len()] == self.string)
            .then_some(self.string)
            .ok_or(ParseError)
    }
}
pub fn string(expected: &str) -> impl Parser<ParseResult = &str> {
    Str { string: expected }
}
