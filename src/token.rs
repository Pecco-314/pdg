use crate::details::WithChar;
use rand::prelude::*;
#[derive(Clone)]
pub enum Token {
    NewLine,
    ConstantInteger(i64),
    RandomIntegerBetween(i64, i64),
    RandomIntegerNoGreaterThan(i64),
    Repeat(usize, Vec<Token>),
}
impl Token {
    pub fn generate(&self) -> String {
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
