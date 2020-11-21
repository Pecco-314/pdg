use colour::e_red;
use std::process::exit;
pub trait Push {
    fn push(&mut self, c: char);
    fn push_str(&mut self, s: &str);
}
pub trait With {
    fn with(mut self, c: char) -> Self
    where
        Self: Push + Sized,
    {
        self.push(c);
        self
    }
    fn with_str(mut self, c: &str) -> Self
    where
        Self: Push + Sized,
    {
        self.push_str(c);
        self
    }
}
impl Push for String {
    fn push(&mut self, c: char) {
        self.push(c);
    }
    fn push_str(&mut self, s: &str) {
        self.push_str(s);
    }
}
impl With for String {}

#[macro_export]
macro_rules! resolve {
    ($t:expr, $ty:ident) => {
        match $t {
            Confirm(i) => i.clone(),
            Lazy(g) => g.generate()?.$ty()?,
        }
    };
    ($t:expr, $ty:ident, $T:ident) => {
        match $t {
            $T::Confirm(i) => i.clone(),
            $T::Lazy(g) => g.generate()?.$ty()?,
        }
    };
}

pub fn error_info(info: &str) -> ! {
    e_red!("error");
    eprint!(": ");
    eprintln!("{}", info);
    exit(1);
}
