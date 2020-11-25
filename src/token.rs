use crate::{
    details::With,
    random::{distribute, random_pair, random_string},
    random_range, resolve,
    token::{Parameter::*, Token::*},
};
use num::ToPrimitive;

#[derive(Clone, Debug)]
pub enum ConfigItem {
    Folder(String),
    Pause(bool),
    Prefix(String),
    Std(String),
}

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub folder: Option<String>,
    pub pause: Option<bool>,
    pub prefix: Option<String>,
    pub std: Option<String>,
}

#[derive(Clone, Debug)]
pub enum RandomString {
    Lower(IntParameter),
    Upper(IntParameter),
    Alpha(IntParameter),
    Bin(IntParameter),
    Oct(IntParameter),
    Dec(IntParameter),
    HexLower(IntParameter),
    HexUpper(IntParameter),
    Alnum(IntParameter),
    Graph(IntParameter),
    OneOf(StrParameter, IntParameter),
    Between(char, char, IntParameter),
}
#[derive(Clone, Debug)]
pub enum RandomInteger {
    Between(IntParameter, IntParameter),
    NoGreaterThan(IntParameter),
}
impl RandomInteger {
    pub fn left(&self) -> IntParameter {
        match self {
            RandomInteger::Between(l, _) => l.clone(),
            RandomInteger::NoGreaterThan(_) => IntParameter::Confirm(0),
        }
    }
    pub fn right(&self) -> IntParameter {
        match self {
            RandomInteger::Between(_, r) => r.clone(),
            RandomInteger::NoGreaterThan(r) => r.clone(),
        }
    }
}
#[derive(Copy, Clone, Debug)]
pub enum Cmp {
    LessThan,
    GreaterThan,
    NoLessThan,
    NoGreaterThan,
}
#[derive(Clone, Debug)]
pub enum Token {
    NewLine,
    ConstantInteger(i64),
    ConstantString(String),
    RandomInteger(RandomInteger),
    RandomString(RandomString),
    TokenGroup(Vec<Token>),
    Repeat(IntParameter, Box<Token>),
    Array(IntParameter, Box<Token>),
    Distribute(Vec<(IntParameter, Token)>),
    RandomIntegerPair(IntParameter, IntParameter, IntParameter, IntParameter, Cmp),
}
#[derive(Clone, Debug)]
pub enum Parameter {
    Int(IntParameter), // Size类型参数当作Int解析再转化
    Char(char),
    Enum(String),
    Str(StrParameter),
    Bool(bool),
}
impl Parameter {
    pub fn int(&self) -> Option<i64> {
        use IntParameter::*;
        match self {
            Int(Confirm(i)) => Some(*i),
            Int(Lazy(i)) => i.generate()?.int(),
            _ => None,
        }
    }
    pub fn str(&self) -> Option<String> {
        use StrParameter::*;
        match self {
            Str(Confirm(s)) => Some(s.clone()),
            Str(Lazy(s)) => s.generate()?.str(),
            _ => None,
        }
    }
}
#[derive(Clone, Debug)]
pub enum IntParameter {
    Confirm(i64),
    Lazy(Box<Token>),
}
#[derive(Clone, Debug)]
pub enum StrParameter {
    Confirm(String),
    Lazy(Box<Token>),
}

impl Token {
    pub fn generate(&self) -> Option<Parameter> {
        use crate::token::{IntParameter::*, RandomInteger::*};
        match self {
            NewLine => Some(Char('\n')),
            ConstantString(s) => Some(Str(StrParameter::Confirm(s.clone()))),
            ConstantInteger(a) => Some(Int(IntParameter::Confirm(*a))),
            RandomInteger(Between(l, r)) => Some(Int(IntParameter::Confirm(random_range!(
                resolve!(l, int),
                resolve!(r, int)
            )))),
            RandomInteger(NoGreaterThan(r)) => Some(Int(IntParameter::Confirm(random_range!(
                0,
                resolve!(r, int)
            )))),
            TokenGroup(v) => {
                let mut s = String::new();
                for i in v.iter() {
                    s.push_str(&i.generate_str()?);
                }
                Some(Str(StrParameter::Confirm(s)))
            }
            Repeat(ip, token) => {
                let mut s = String::new();
                let times = resolve!(ip, size);
                for _ in 0..times {
                    s.push_str(&token.generate_str()?);
                }
                Some(Str(StrParameter::Confirm(s)))
            }
            Array(ip, v) => Some(Str(StrParameter::Confirm({
                let times = resolve!(ip, size);
                times.to_string().with('\n')
                    + Repeat(ip.clone(), v.clone()).generate_str()?.as_str()
            }))),
            Distribute(v) => {
                let mut v2 = Vec::new();
                for (ip, token) in v.iter() {
                    v2.push((resolve!(ip, size), token));
                }
                let token = distribute(v2)?;
                token.generate()
            }
            RandomIntegerPair(l1, r1, l2, r2, op) => {
                let (a, b) = random_pair(
                    resolve!(l1, int),
                    resolve!(r1, int),
                    resolve!(l2, int),
                    resolve!(r2, int),
                    *op,
                );
                let mut s = a.to_string().with(' ');
                s.push_str(&b.to_string());
                Some(Str(StrParameter::Confirm(s.with(' '))))
            }
            RandomString(rs) => Some(Str(StrParameter::Confirm(random_string(&rs)?))),
        }
    }
    pub fn generate_str(&self) -> Option<String> {
        match self.generate()? {
            Parameter::Int(IntParameter::Confirm(i)) => Some(i.to_string().with(' ')),
            Parameter::Char(c) => Some(c.to_string()),
            Parameter::Str(StrParameter::Confirm(s)) => Some(s),
            _ => None,
        }
    }
}
