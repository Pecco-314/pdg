use crate::combinator::*;
use crate::{ParseError, Parser};
use num::traits::FromPrimitive;

/// 解析指定字符
pub fn char(expected: char) -> impl Parser<ParseResult = char> {
    satisfy(move |c| c == expected)
}

/// 解析任意字符
pub fn any() -> impl Parser<ParseResult = char> {
    satisfy(|_| true)
}

/// 解析十进制数字，返回字符
pub fn digit() -> impl Parser<ParseResult = char> {
    satisfy(|c: char| c.is_digit(10))
}

/// 解析拉丁字母
pub fn alpha() -> impl Parser<ParseResult = char> {
    satisfy(|c: char| c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z')
}

/// 解析空白符
pub fn space() -> impl Parser<ParseResult = char> {
    satisfy(|c: char| c.is_whitespace())
}

/// 解析任意数量空白符，返回()
pub fn spaces() -> impl Parser<ParseResult = ()> {
    many(ignore(space()))
}

/// 解析所给字符串中的任意字符
pub fn one_of(s: &str) -> impl Parser<ParseResult = char> + '_ {
    satisfy(move |c| s.contains(c))
}

/// 解析数字（浮点数）
pub fn number() -> impl Parser<ParseResult = f64> {
    many1(one_of("0123456789Ee-.")).flat_map(|s: String| s.parse::<f64>())
}

/// 解析能转化成某种数字类型的数字
pub fn into_num<I>() -> impl Parser<ParseResult = I>
where
    I: FromPrimitive + std::fmt::Debug,
{
    const EPS: f64 = 1e-10;
    number().flat_map(|x| {
        if (x - x.trunc()).abs() < EPS {
            // 小数部分足够小则解析成功
            I::from_f64(x).ok_or(ParseError)
        } else {
            Err(ParseError)
        }
    })
}

/// 解析usize
pub fn size() -> impl Parser<ParseResult = usize> {
    many1(one_of("0123456789")).flat_map(|s: String| s.parse::<usize>())
}

#[derive(Copy, Clone)]
pub struct Str<'a> {
    string: &'a str,
}
impl<'a> Parser for Str<'a> {
    type ParseResult = &'a str;
    fn parse(&self, buf: &mut &str) -> Result<Self::ParseResult, ParseError> {
        let len = self.string.len();
        if buf.len() >= len && &buf[..len] == self.string {
            *buf = &buf[len..];
            Ok(self.string)
        } else {
            Err(ParseError)
        }
    }
}
pub fn string(expected: &str) -> impl Parser<ParseResult = &str> {
    Str { string: expected }
}
