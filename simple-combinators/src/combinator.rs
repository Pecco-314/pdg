use crate::{slice_some, IntoParseError, ParseError, Parser};
use std::marker::PhantomData;

macro_rules! impl_copy_and_clone {
    ($struct:ident<$($ctype:ident),*; $($ptype:ident),*>) => {
        impl<$($ctype),*, $($ptype),*> Copy for $struct<$($ctype),* ,$($ptype),*> where $($ctype : Copy),* {}
        impl<$($ctype),* ,$($ptype),*> Clone for $struct<$($ctype),* ,$($ptype),*>
        where
        $($ctype : Copy),*
        {
            fn clone(&self) -> Self {
                *self
            }
        }
    };
} // 为无法自动derive Copy & Clone的结构体实现Copy & Clone

#[derive(Clone, Copy)]
pub struct Satisfy<F> {
    satisfy_func: F,
}

impl<F> Parser for Satisfy<F>
where
    F: Fn(char) -> bool + Copy,
{
    type ParseResult = char;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let mut iter = buf.chars();
        let first = iter.next().ok_or(ParseError {
            position: slice_some(buf),
        })?;
        let res = (self.satisfy_func)(first)
            .then_some(first)
            .ok_or(ParseError {
                position: slice_some(buf),
            })?;
        *buf = iter.as_str();
        Ok(res)
    }
}

/// 匹配符合条件的字符
pub fn satisfy<F>(satisfy_func: F) -> Satisfy<F> {
    Satisfy { satisfy_func }
}
#[derive(Copy, Clone)]
pub struct With<P1, P2> {
    pub(crate) parser1: P1,
    pub(crate) parser2: P2,
}

impl<P1, P2> Parser for With<P1, P2>
where
    P1: Parser,
    P2: Parser,
{
    type ParseResult = P2::ParseResult;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        self.parser1.parse(buf)?;
        let res = self.parser2.parse(buf)?;
        Ok(res)
    }
}

#[derive(Copy, Clone)]
pub struct Skip<P1, P2> {
    pub(crate) parser1: P1,
    pub(crate) parser2: P2,
}

impl<P1, P2> Parser for Skip<P1, P2>
where
    P1: Parser,
    P2: Parser,
{
    type ParseResult = P1::ParseResult;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let res = self.parser1.parse(buf)?;
        self.parser2.parse(buf)?;
        Ok(res)
    }
}

#[derive(Copy, Clone)]
pub struct And<P1, P2> {
    pub(crate) parser1: P1,
    pub(crate) parser2: P2,
}

impl<P1, P2> Parser for And<P1, P2>
where
    P1: Parser,
    P2: Parser,
{
    type ParseResult = (P1::ParseResult, P2::ParseResult);
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let res1 = self.parser1.parse(buf)?;
        let res2 = self.parser2.parse(buf)?;
        Ok((res1, res2))
    }
}

#[derive(Copy, Clone)]
pub struct Map<P, F> {
    pub(crate) parser: P,
    pub(crate) callback: F,
}
impl<P, F, R> Parser for Map<P, F>
where
    P: Parser,
    F: Fn(P::ParseResult) -> R + Copy,
{
    type ParseResult = R;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let raw = self.parser.parse(buf)?;
        Ok((self.callback)(raw))
    }
}

pub struct FlatMap<P, F, R> {
    pub(crate) parser: P,
    pub(crate) callback: F,
    pub(crate) mark: PhantomData<R>,
}
impl_copy_and_clone!(FlatMap<P,F;R>);
impl<P, F, R, E> Parser for FlatMap<P, F, R>
where
    P: Parser,
    F: Fn(P::ParseResult) -> E + Copy,
    E: IntoParseError<R>,
{
    type ParseResult = R;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let raw = self.parser.parse(buf)?;
        let res = (self.callback)(raw).into_if_err(ParseError {
            position: slice_some(buf),
        })?;
        Ok(res)
    }
}

#[derive(Copy, Clone)]
pub struct Attempt<P> {
    parser: P,
}
impl<P> Parser for Attempt<P>
where
    P: Parser,
{
    type ParseResult = P::ParseResult;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let backup = *buf;
        let res = self.parser.parse(buf);
        if res.is_err() {
            *buf = backup;
        }
        res
    }
}
pub fn attempt<P>(parser: P) -> Attempt<P>
where
    P: Parser,
{
    Attempt { parser: parser }
}

#[derive(Copy, Clone)]
pub struct Optional<P> {
    parser: P,
}
impl<P> Parser for Optional<P>
where
    P: Parser,
{
    type ParseResult = Option<P::ParseResult>;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let backup = *buf;
        let res = self.parser.parse(buf);
        if res.is_err() {
            *buf = backup;
        }
        Ok(res.ok())
    }
}
pub fn optional<P>(parser: P) -> Optional<P>
where
    P: Parser,
{
    Optional { parser: parser }
}

#[derive(Copy, Clone)]
pub struct Preview<P> {
    parser: P,
}
impl<P> Parser for Preview<P>
where
    P: Parser,
{
    type ParseResult = P::ParseResult;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let backup = *buf;
        let res = self.parser.parse(buf);
        *buf = backup;
        res
    }
}
pub fn preview<P>(parser: P) -> Preview<P>
where
    P: Parser,
{
    Preview { parser: parser }
}

pub fn ignore<P>(parser: P) -> Map<P, impl Fn(P::ParseResult) + Copy>
where
    P: Parser,
{
    parser.map(|_| ())
}

pub struct Repeat<P, R> {
    pub(crate) parser: P,
    pub(crate) times: usize,
    pub(crate) output: PhantomData<R>,
}
impl<P, R> Copy for Repeat<P, R> where P: Parser {}
impl<P, R> Clone for Repeat<P, R>
where
    P: Parser,
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<P, R> Parser for Repeat<P, R>
where
    P: Parser,
    R: Extend<P::ParseResult> + Default,
{
    type ParseResult = R;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let backup = *buf;
        let mut iter = self.parser.iter(buf);
        let mut collection = R::default();
        for _ in 0..self.times {
            let cur = iter.next().ok_or(ParseError {
                position: slice_some(backup),
            })?;
            collection.extend_one(cur);
        }
        Ok(collection)
    }
}

pub struct Many<P, R> {
    pub(crate) parser: P,
    pub(crate) output: PhantomData<R>,
}
impl_copy_and_clone!(Many<P;R>);
impl<P, R> Parser for Many<P, R>
where
    P: Parser,
    R: Extend<P::ParseResult> + Default,
{
    type ParseResult = R;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let mut collection = R::default();
        collection.extend(self.parser.iter(buf));
        Ok(collection)
    }
}
pub fn many<P, R>(parser: P) -> Many<P, R>
where
    P: Parser,
    R: Extend<P::ParseResult> + Default,
{
    Many {
        parser: parser,
        output: PhantomData,
    }
}

pub struct Many1<P, R> {
    pub(crate) parser: P,
    pub(crate) output: PhantomData<R>,
}
impl_copy_and_clone!(Many1<P;R>);
impl<P, R> Parser for Many1<P, R>
where
    P: Parser,
    R: Extend<P::ParseResult> + Default,
{
    type ParseResult = R;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let mut collection = R::default();
        let backup = *buf;
        let mut iter = self.parser.iter(buf);
        let first = iter.next().ok_or(ParseError {
            position: slice_some(backup),
        })?;
        collection.extend_one(first);
        collection.extend(iter);
        Ok(collection)
    }
}
pub fn many1<P, R>(parser: P) -> Many1<P, R>
where
    P: Parser,
    R: Extend<P::ParseResult> + Default,
{
    Many1 {
        parser: parser,
        output: PhantomData,
    }
}

pub struct SepBy<P1, P2, R> {
    pub(crate) parser: P1,
    pub(crate) sep: P2,
    pub(crate) output: PhantomData<R>,
}
impl_copy_and_clone!(SepBy<P1,P2;R>);
impl<P1, P2, R> Parser for SepBy<P1, P2, R>
where
    P1: Parser,
    P2: Parser,
    R: Extend<P1::ParseResult> + Default,
{
    type ParseResult = R;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let mut collection = R::default();
        let mut iter = self.parser.iter(buf);
        let first = iter.next().ok_or(ParseError {
            position: slice_some(buf),
        })?;
        collection.extend_one(first);
        let sep = attempt(self.sep.with(self.parser));
        let iter = sep.iter(buf);
        collection.extend(iter);
        Ok(collection)
    }
}

#[derive(Copy, Clone)]
pub struct Or<P1, P2> {
    pub(crate) parser1: P1,
    pub(crate) parser2: P2,
}
impl<P1, P2, R> Parser for Or<P1, P2>
where
    P1: Parser<ParseResult = R>,
    P2: Parser<ParseResult = R>,
{
    type ParseResult = R;
    fn parse<'a>(&self, buf: &mut &'a str) -> Result<Self::ParseResult, ParseError<'a>> {
        let res = attempt(self.parser1).parse(buf);
        if res.is_ok() {
            res
        } else {
            self.parser2.parse(buf)
        }
    }
}
