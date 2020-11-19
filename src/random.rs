use crate::token::*;
use rand::prelude::*;
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
    // unimplemented!();
}
