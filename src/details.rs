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
