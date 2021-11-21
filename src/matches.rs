pub trait Matcher<I>
where
    I: Iterator,
{
    type Matched;
    fn check(&self, iter: &mut I) -> bool {
        self.r#match(iter).is_some()
    }
    fn r#match(&self, iter: &mut I) -> Option<Self::Matched>;
    fn matches<'a>(&'a self, iter: &'a mut I) -> Matches<'a, Self, I>
    where
        Self: Sized,
    {
        Matches::new(self, iter)
    }
}

pub struct Matches<'a, M, I>
where
    I: Iterator,
{
    matcher: &'a M,
    iter: &'a mut I,
}

impl<'a, M, I> Matches<'a, M, I>
where
    M: Matcher<I>,
    I: Iterator,
{
    pub fn new(matcher: &'a M, iter: &'a mut I) -> Self {
        Self { matcher, iter }
    }
}

impl<'a, M, I> Iterator for Matches<'a, M, I>
where
    M: Matcher<I>,
    I: Iterator,
{
    type Item = M::Matched;
    fn next(&mut self) -> Option<Self::Item> {
        if self.iter.size_hint().0 > 0 {
            self.matcher.r#match(self.iter)
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (1, self.iter.size_hint().1)
    }
}
