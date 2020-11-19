use crate::details::With;
use crate::random::random_pair;
use rand::prelude::*;
pub use Gen::*;
pub use Op::*;

#[derive(Clone)]
pub enum Gen {
    NewLine,
    ConstantInteger(i64),
    RandomIntegerBetween(i64, i64),
    RandomIntegerNoGreaterThan(i64),
    Repeat(usize, Vec<Token>),
    RandomPair(i64, i64, i64, i64, Op),
}
#[derive(Copy, Clone)]
pub enum Op {
    LessThan,
    GreaterThan,
    NoLessThan,
    NoGreaterThan,
}
#[derive(Clone)]
pub enum Token {
    Gen(Gen),
    Op(Op),
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
    Some(RandomPair(l1, r1, l2, r2, *op))
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
    pub fn generate(&self) -> Option<String> {
        match self {
            NewLine => Some('\n'.to_string()),
            ConstantInteger(a) => Some(a.to_string().with(' ')),
            RandomIntegerBetween(a, b) => {
                Some(thread_rng().gen_range(*a, *b + 1).to_string().with(' '))
            }
            RandomIntegerNoGreaterThan(a) => {
                Some(thread_rng().gen_range(0, *a + 1).to_string().with(' '))
            }
            Repeat(times, v) => {
                let mut s = String::new();
                let mut gens = cul_token(v)?;
                for _ in 0..*times {
                    for i in gens.iter_mut() {
                        s.push_str(&i.generate()?);
                    }
                }
                Some(s)
            }
            RandomPair(l1, r1, l2, r2, op) => {
                let (a, b) = random_pair(*l1, *r1, *l2, *r2, *op);
                let mut s = a.to_string().with(' ');
                s.push_str(&b.to_string());
                Some(s.with(' '))
            }
        }
    }
}
