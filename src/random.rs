use crate::token::*;
use rand::{seq::SliceRandom, thread_rng, Rng};
macro_rules! distribute{
    ($sum:expr; $($freq:expr, $func:expr);+)=>{{
            let mut ans = None;
            let mut cc = thread_rng().gen_range(0,$sum);
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

pub fn random_pair(l1: i64, r1: i64, l2: i64, r2: i64, op: Op) -> (i64, i64) {
    match op {
        LessThan => {
            let (x, y) = random_pair(l1, r1, l2 - 1, r2 - 1, NoGreaterThan);
            (x, y + 1)
        }
        GreaterThan => {
            let (x, y) = random_pair(l1, r1, l2 + 1, r2 + 1, NoLessThan);
            (x, y - 1)
        }
        NoGreaterThan => {
            let r1 = r1.min(r2);
            let l2 = l2.max(l1);
            loop {
                let x = thread_rng().gen_range(l1, r1 + 1);
                let y = thread_rng().gen_range(l2, r2 + 1);
                if x <= y {
                    return (x, y);
                }
            }
        }
        NoLessThan => {
            let r2 = r1.min(r2);
            let l1 = l2.max(l1);
            loop {
                let x = thread_rng().gen_range(l1, r1 + 1);
                let y = thread_rng().gen_range(l2, r2 + 1);
                if x >= y {
                    return (x, y);
                }
            }
        }
    }
}

pub fn random_char(l: char, r: char) -> Option<char> {
    std::char::from_u32(thread_rng().gen_range(l as u32, r as u32 + 1))
}

pub fn random_string(rs: &RandomString) -> Option<String> {
    use RandomString::*;
    let mut s = String::new();
    match rs {
        OneOf(times, dict) => {
            let dict: Vec<char> = dict.chars().collect();
            for _ in 0..*times {
                s.push(*dict[..].choose(&mut thread_rng())?);
            }
            Some(s)
        }
        Alpha(times) => {
            for _ in 0..*times {
                s.push(
                    distribute!(52; 26, || random_char('a', 'z'); 26, || random_char('A', 'Z'))?,
                );
            }
            Some(s)
        }
        Alnum(times) => {
            for _ in 0..*times {
                s.push(
                    distribute!(62; 26, || random_char('a', 'z'); 26, || random_char('A', 'Z'); 10, || random_char('0','9'))?,
                );
            }
            Some(s)
        }
        HexLower(times) => {
            for _ in 0..*times {
                s.push(distribute!(16; 10, || random_char('0', '9'); 6, || random_char('a', 'f'))?);
            }
            Some(s)
        }
        HexUpper(times) => {
            for _ in 0..*times {
                s.push(distribute!(16; 10, || random_char('0', '9'); 6, || random_char('A', 'F'))?);
            }
            Some(s)
        }
        Within(times, l, r) => {
            for _ in 0..*times {
                s.push(random_char(*l, *r)?);
            }
            Some(s)
        }
        Lower(times) => random_string(&Within(*times, 'a', 'z')),
        Upper(times) => random_string(&Within(*times, 'A', 'Z')),
        Bin(times) => random_string(&Within(*times, '0', '1')),
        Oct(times) => random_string(&Within(*times, '0', '7')),
        Dec(times) => random_string(&Within(*times, '0', '9')),
        Graph(times) => random_string(&Within(*times, '!', '~')),
    }
}
