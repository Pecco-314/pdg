mod details;
use details::WithChar;
use rand::prelude::*;
use simple_combinators::{
    combinator::{attempt, many1, optional},
    parser::{char, into_integer, spaces},
    Parser, ParserIter,
};

enum Token {
    NewLine,
    ConstantInteger(i64),
    RandomIntegerBetween(i64, i64),
    RandomIntegerNoGreaterThan(i64),
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
        }
    }
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
                .or(char('/').map(|_| Token::NewLine)),
        )
        .skip(spaces())
}

fn main() {
    let filename = "data/template.txt";
    let template = std::fs::read_to_string(filename).expect("failed to read");
    let mut buf = template.as_str();
    let token = token();
    let iter = token.iter(&mut buf).map(|x| x.generate());
    let s: String = iter.collect();
    println!("{}", s);
}
