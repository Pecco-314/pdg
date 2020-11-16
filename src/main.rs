mod details;
use details::WithChar;
use rand::prelude::*;
use simple_combinators::{
    combinator::optional,
    parser::{char, into_integer, spaces},
    ParseError, Parser,
};

enum Token {
    NewLine,
    ConstantInteger(i64),
    RandomIntegerBetween(i64, i64),
    RandomIntegerNoGreaterThan(i64),
    Repeat(usize, Vec<Token>),
}
impl Token {
    fn generate(&self) -> String {
        match self {
            Token::NewLine => '\n'.to_string(),
            Token::ConstantInteger(a) => a.to_string().with(' '),
            Token::RandomIntegerBetween(a, b) => thread_rng().gen_range(a, b).to_string().with(' '),
            Token::RandomIntegerNoGreaterThan(a) => {
                thread_rng().gen_range(0, a).to_string().with(' ')
            }
            Token::Repeat(times, v) => {
                let mut s = String::new();
                for _ in 0..*times {
                    for i in v.iter() {
                        s.push_str(&i.generate());
                    }
                }
                s
            }
        }
    }
}

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

fn token() -> impl Parser<ParseResult = Token> {
    spaces()
        .with(
            random_integer_token()
                .or(into_integer().map(|i| Token::ConstantInteger(i)))
                .or(repeated_token())
                .or(char('/').map(|_| Token::NewLine)),
        )
        .skip(spaces())
}

fn main() {
    let template = "data/template.txt";
    let output = "data/output.txt";
    let template = std::fs::read_to_string(template).expect("Failed to read");
    let mut buf = template.as_str();
    let token = token();
    let iter = token.iter(&mut buf).map(|x| x.generate());
    let s: String = iter.collect();
    std::fs::write(output, s).expect("Failed to write");
}
