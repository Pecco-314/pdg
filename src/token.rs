use self::{Gen::*, IntParameter::*, Parameter::*};
use crate::{
    details::With,
    random::{random_pair, random_string},
};
use rand::prelude::{thread_rng, Rng};

#[derive(Clone, Debug)]
pub enum ConfigItem {
    Fold(String),
    Pause(bool),
    Prefix(String),
}

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub fold: Option<String>,
    pub pause: Option<bool>,
    pub prefix: Option<String>,
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
    OneOf(String, IntParameter),
    Between(char, char, IntParameter),
}
#[derive(Clone, Debug)]
pub enum Gen {
    NewLine,
    ConstantInteger(i64),
    ConstantString(String),
    RandomIntegerBetween(i64, i64),
    RandomIntegerNoGreaterThan(i64),
    Repeat(usize, Vec<Token>),
    Array(usize, Vec<Token>),
    RandomIntegerPair(i64, i64, i64, i64, Op),
    RandomString(RandomString),
}
#[derive(Copy, Clone, Debug)]
pub enum Op {
    LessThan,
    GreaterThan,
    NoLessThan,
    NoGreaterThan,
}
#[derive(Clone, Debug)]
pub enum Token {
    Gen(Gen),
    Op(Op),
}
#[derive(Clone, Debug)]
pub enum Parameter {
    Int(IntParameter), // Size类型参数当作Int解析再转化
    Char(char),
    Enum(String),
    Str(String),
    Bool(bool),
}
impl Parameter {
    pub fn int(&self) -> Option<i64> {
        match self {
            Parameter::Int(Confirm(i)) => Some(*i),
            Parameter::Int(Lazy(i)) => i.generate()?.int(),
            _ => None,
        }
    }
}
#[derive(Clone, Debug)]
pub enum IntParameter {
    Confirm(i64),
    Lazy(Box<Gen>),
}

fn cul(a: &Gen, b: &Gen, op: &Op) -> Option<Gen> {
    let (l1, r1) = match a {
        RandomIntegerBetween(l, r) => (*l, *r),
        RandomIntegerNoGreaterThan(r) => (0, *r),
        _ => return None,
    };
    let (l2, r2) = match b {
        RandomIntegerBetween(l, r) => (*l, *r),
        RandomIntegerNoGreaterThan(r) => (0, *r),
        _ => return None,
    };
    Some(RandomIntegerPair(l1, r1, l2, r2, *op))
}
pub fn cul_token(tokens: &Vec<Token>) -> Option<Vec<Gen>> {
    let mut gens = Vec::new();
    let mut cur_op = None;
    for t in tokens.iter() {
        match t {
            Token::Gen(gen) => {
                if let Some(op) = cur_op {
                    let ind = gens.len() - 1;
                    gens[ind] = cul(gens.last()?, gen, op)?;
                    cur_op = None;
                } else {
                    gens.push(gen.clone());
                }
            }
            Token::Op(op) => {
                cur_op = Some(op);
            }
        }
    }
    Some(gens)
}
impl Gen {
    pub fn generate(&self) -> Option<Parameter> {
        match self {
            NewLine => Some(Char('\n')),
            ConstantString(s) => Some(Str(s.clone())),
            ConstantInteger(a) => Some(Int(Confirm(*a))),
            RandomIntegerBetween(a, b) => Some(Int(Confirm(thread_rng().gen_range(*a, *b + 1)))),
            RandomIntegerNoGreaterThan(a) => Some(Int(Confirm(thread_rng().gen_range(0, *a + 1)))),
            Repeat(times, v) => {
                let mut s = String::new();
                let mut gens = cul_token(v)?;
                for _ in 0..*times {
                    for i in gens.iter_mut() {
                        s.push_str(&i.generate_str()?);
                    }
                }
                Some(Str(s))
            }
            Array(times, v) => {
                Some(Str(times.to_string().with('\n')
                    + Repeat(*times, v.clone()).generate_str()?.as_str()))
            }
            RandomIntegerPair(l1, r1, l2, r2, op) => {
                let (a, b) = random_pair(*l1, *r1, *l2, *r2, *op);
                let mut s = a.to_string().with(' ');
                s.push_str(&b.to_string());
                Some(Str(s.with(' ')))
            }
            RandomString(rs) => Some(Str(random_string(&rs)?)),
        }
    }
    pub fn generate_str(&self) -> Option<String> {
        match self.generate()? {
            Parameter::Int(Confirm(i)) => Some(i.to_string().with(' ')),
            Parameter::Char(c) => Some(c.to_string()),
            Parameter::Str(s) => Some(s),
            _ => None,
        }
    }
}
