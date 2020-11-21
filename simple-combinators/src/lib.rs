#![feature(bool_to_option)]
#![feature(try_trait)]
#![feature(extend_one)]
pub mod combinator;
pub mod parser;
use combinator::*;
use derive_more::{Display, Error};
use std::marker::PhantomData;
use std::option::NoneError;

#[derive(Debug, Display, Error)]
pub struct ParseError;
impl From<NoneError> for ParseError {
    fn from(_: NoneError) -> ParseError {
        ParseError
    }
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

pub trait Parser: Copy + Clone {
    type ParseResult;
    fn parse(&self, buf: &mut &str) -> Result<Self::ParseResult, ParseError>;
    fn iter<'a, 'b>(&'a self, buf: &'a mut &'b str) -> ParserIter<'a, 'b, Self> {
        ParserIter {
            parser: self,
            buf: buf,
        }
    }
    fn with<P>(self, other: P) -> With<Self, P> {
        With {
            parser1: self,
            parser2: other,
        }
    }
    fn skip<P>(self, other: P) -> Skip<Self, P> {
        Skip {
            parser1: self,
            parser2: other,
        }
    }
    fn and<P>(self, other: P) -> And<Self, P> {
        And {
            parser1: self,
            parser2: other,
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
    fn flat_map<F, R, E>(self, callback: F) -> FlatMap<Self, F>
    where
        F: Fn(Self::ParseResult) -> Result<R, E>,
    {
        FlatMap {
            parser: self,
            callback: callback,
        }
    }
    fn repeat<P>(self, times: usize) -> Repeat<Self, P> {
        Repeat {
            parser: self,
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
            parser1: self,
            parser2: other,
        }
    }
}
