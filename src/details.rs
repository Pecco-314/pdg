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
    ($t:expr, size) => {
        resolve!($t, int).to_usize()?
    };
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

pub trait Ignore<T> {
    fn ignore(self) -> T; // 默认不会panic的情况下忽略错误
}

impl<T, E> Ignore<T> for Result<T, E> {
    fn ignore(self) -> T {
        self.unwrap_or_else(|_| panic!(""))
    }
}

impl<T> Ignore<T> for Option<T> {
    fn ignore(self) -> T {
        self.unwrap_or_else(|| panic!(""))
    }
}

pub fn error_info(info: &str) -> ! {
    e_red!("error");
    eprint!(": ");
    eprintln!("{}", info);
    exit(1);
}
