use crate::token::Token;
use simple_combinators::{
    combinator::optional,
    parser::{char, into_integer, spaces},
    ParseError, Parser,
};
#[derive(Copy, Clone)]
struct RepeatedToken;
impl Parser for RepeatedToken {
    type ParseResult = Token;
    fn parse(&self, buf: &mut &str) -> Result<Self::ParseResult, ParseError> {
        let times = char('x')
            .with(into_integer())
            .skip(spaces())
            .skip(char('{'))
            .parse(buf)? as usize;
        let mut v = Vec::new();
        for i in token().iter(buf) {
            v.push(i);
        }
        char('}').parse(buf)?;
        Ok(Token::Repeat(times, v))
    }
}
fn repeated_token() -> RepeatedToken {
    RepeatedToken
}

fn random_integer_token() -> impl Parser<ParseResult = Token> {
    char('i')
        .with(into_integer())
        .and(optional(char(',').with(into_integer())))
        .map(|(a, opt)| match opt {
            Some(b) => Token::RandomIntegerBetween(a, b),
            None => Token::RandomIntegerNoGreaterThan(a),
        })
}

pub fn token() -> impl Parser<ParseResult = Token> {
    spaces()
        .with(
            random_integer_token()
                .or(into_integer().map(|i| Token::ConstantInteger(i)))
                .or(repeated_token())
                .or(char('/').map(|_| Token::NewLine)),
        )
        .skip(spaces())
}
