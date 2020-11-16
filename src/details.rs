pub trait Push {
    fn push(&mut self, c: char);
}
pub trait WithChar {
    fn with(mut self, c: char) -> Self
    where
        Self: Push + Sized,
    {
        self.push(c);
        self
    }
}
impl Push for String {
    fn push(&mut self, c: char) {
        self.push(c);
    }
}
impl WithChar for String {}
