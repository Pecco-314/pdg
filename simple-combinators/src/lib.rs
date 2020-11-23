#![feature(bool_to_option)]
#![feature(extend_one)]
pub mod combinator;
pub mod parser;
use combinator::*;
use std::{fmt::Debug, marker::PhantomData};

#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub struct ParseError<'a> {
    pub position: &'a str,
}

pub trait IntoParseError<T> {
    fn into_if_err<'a>(self, err: ParseError<'a>) -> Result<T, ParseError<'a>>;
}
impl<T> IntoParseError<T> for Option<T> {
    fn into_if_err<'a>(self, err: ParseError<'a>) -> Result<T, ParseError<'a>> {
        self.ok_or(err)
    }
}
impl<T, E> IntoParseError<T> for Result<T, E> {
    fn into_if_err<'a>(self, err: ParseError<'a>) -> Result<T, ParseError<'a>> {
        self.map_err(|_| err)
    }
}
pub fn slice_some(buf: &str) -> &str {
    for (i, c) in buf.char_indices() {
        if c == '\n' {
            return &buf[..i];
        }
    }
    &buf[..]
}

pub struct ParserIter<'a, 'b, P> {
    pub(crate) parser: &'a P,
    pub(crate) buf: &'a mut &'b str,
}
impl<P> Iterator for ParserIter<'_, '_, P>
where
    P: Parser,
{
    type Item = P::ParseResult;
    fn next(&mut self) -> Option<Self::Item> {
        self.parser.parse(self.buf).ok()
    }
}
impl<'a, 'b, P> ParserIter<'a, 'b, P> {
    pub fn with_result(self) -> ParseResultIter<'a, 'b, P> {
        ParseResultIter {
            parser: self.parser,
            buf: self.buf,
            end: false,
        }
    }
}
pub struct ParseResultIter<'a, 'b, P> {
    pub(crate) parser: &'a P,
    pub(crate) buf: &'a mut &'b str,
    pub(crate) end: bool,
}
impl<'a, 'b, P> Iterator for ParseResultIter<'a, 'b, P>
where
    P: Parser,
{
    type Item = Result<P::ParseResult, ParseError<'b>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }
        let res = self.parser.parse(self.buf);
        if res.is_err() {
            self.end = true;
        }
        Some(res)
    }
}

pub trait Parser: Copy + Clone {
    type ParseResult;
    fn parse<'a, 'b>(&'a self, buf: &'a mut &'b str) -> Result<Self::ParseResult, ParseError<'b>>;
    fn iter<'a, 'b>(&'a self, buf: &'a mut &'b str) -> ParserIter<'a, 'b, Self> {
        ParserIter {
            parser: self,
            buf: buf,
        }
    }
    fn with<P>(self, other: P) -> With<Self, P> {
        With {
            skip: self,
            with: other,
        }
    }
    fn skip<P>(self, other: P) -> Skip<Self, P> {
        Skip {
            with: self,
            skip: other,
        }
    }
    fn and<P>(self, other: P) -> And<Self, P> {
        And {
            and1: self,
            and2: other,
        }
    }
    fn between<L, R>(self, left: L, right: R) -> Skip<With<L, Self>, R>
    where
        L: Parser,
        R: Parser,
    {
        left.with(self).skip(right)
    }

    fn map<F, R>(self, callback: F) -> Map<Self, F>
    where
        F: Fn(Self::ParseResult) -> R,
    {
        Map {
            parser: self,
            callback: callback,
        }
    }
    fn flat_map<F, R, E>(self, callback: F) -> FlatMap<Self, F, R>
    where
        F: Fn(Self::ParseResult) -> E + Copy,
    {
        FlatMap {
            parser: self,
            callback: callback,
            mark: PhantomData,
        }
    }
    fn repeat<P>(self, times: usize) -> Repeat<Self, P> {
        Repeat {
            repeat: self,
            times: times,
            output: PhantomData,
        }
    }
    fn sep_by<P, R>(self, sep: P) -> SepBy<Self, P, R> {
        SepBy {
            parser: self,
            sep: sep,
            output: PhantomData,
        }
    }
    fn or<P>(self, other: P) -> Or<Self, P> {
        Or {
            branch1: self,
            branch2: other,
        }
    }
}
