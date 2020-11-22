use crate::{
    resolve,
    token::{Op, RandomString, RandomString::*, *},
};
use num::cast::ToPrimitive;
use rand::{seq::SliceRandom, thread_rng};
macro_rules! distribute{
    ($sum:expr; $($freq:expr, $func:expr);+)=>{{
            let mut ans = None;
            let mut cc = random_range!(0,$sum);
            'lp:{$(
                cc -= $freq;
                if cc<0 {
                    ans = Some($func()?);
                    break 'lp;
                }
            )+
            }
            ans
    }}
}

#[macro_export]
macro_rules! random_range {
    ($a:expr, $b:expr) => {
        if $a > $b {
            use crate::details::error_info;
            error_info(&format!(
                "Tried to generate random integer between {} and {}, but {} > {}",
                $a, $b, $a, $b,
            ))
        } else {
            use rand::prelude::{thread_rng, Rng};
            thread_rng().gen_range($a, $b + 1)
        }
    };
}

pub fn random_pair(l1: i64, r1: i64, l2: i64, r2: i64, op: Op) -> (i64, i64) {
    match op {
        Op::LessThan => {
            let (x, y) = random_pair(l1, r1, l2 - 1, r2 - 1, Op::NoGreaterThan);
            (x, y)
        }
        Op::GreaterThan => {
            let (x, y) = random_pair(l1, r1, l2 + 1, r2 + 1, Op::NoLessThan);
            (x, y)
        }
        Op::NoGreaterThan => {
            let r1 = r1.min(r2);
            let l2 = l2.max(l1);
            loop {
                let x = random_range!(l1, r1);
                let y = random_range!(l2, r2);
                if x <= y {
                    return (x, y);
                }
            }
        }
        Op::NoLessThan => {
            let r2 = r1.min(r2);
            let l1 = l2.max(l1);
            loop {
                let x = random_range!(l1, r1);
                let y = random_range!(l2, r2);
                if x >= y {
                    return (x, y);
                }
            }
        }
    }
}

pub fn random_char(l: char, r: char) -> Option<char> {
    std::char::from_u32(random_range!(l as u32, r as u32 + 1))
}

pub fn random_string(rs: &RandomString) -> Option<String> {
    use IntParameter::*;
    let mut s = String::new();
    match rs {
        OneOf(dict, t) => {
            let dict: Vec<char> = resolve!(dict, str, StrParameter).chars().collect();
            for _ in 0..resolve!(t, int).to_usize()? {
                s.push(*dict[..].choose(&mut thread_rng())?);
            }
            Some(s)
        }
        Alpha(t) => {
            for _ in 0..resolve!(t, int).to_usize()? {
                s.push(
                    distribute!(52; 26, || random_char('a', 'z'); 26, || random_char('A', 'Z'))?,
                );
            }
            Some(s)
        }
        Alnum(t) => {
            for _ in 0..resolve!(t, int).to_usize()? {
                s.push(
                    distribute!(62; 26, || random_char('a', 'z'); 26, || random_char('A', 'Z'); 10, || random_char('0','9'))?,
                );
            }
            Some(s)
        }
        HexLower(t) => {
            for _ in 0..resolve!(t, int).to_usize()? {
                s.push(distribute!(16; 10, || random_char('0', '9'); 6, || random_char('a', 'f'))?);
            }
            Some(s)
        }
        HexUpper(t) => {
            for _ in 0..resolve!(t, int).to_usize()? {
                s.push(distribute!(16; 10, || random_char('0', '9'); 6, || random_char('A', 'F'))?);
            }
            Some(s)
        }
        Between(l, r, t) => {
            for _ in 0..resolve!(t, int).to_usize()? {
                s.push(random_char(*l, *r)?);
            }
            Some(s)
        }
        Lower(t) => random_string(&Between('a', 'z', t.clone())),
        Upper(t) => random_string(&Between('A', 'Z', t.clone())),
        Bin(t) => random_string(&Between('0', '1', t.clone())),
        Oct(t) => random_string(&Between('0', '7', t.clone())),
        Dec(t) => random_string(&Between('0', '9', t.clone())),
        Graph(t) => random_string(&Between('!', '~', t.clone())),
    }
}
