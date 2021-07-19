pub trait IterExt: Iterator {
    fn split_by<F>(&mut self, split: F) -> SplitBy<'_, Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool;
}

impl<I: Iterator> IterExt for I {
    fn split_by<F>(&mut self, split: F) -> SplitBy<'_, Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool,
    {
        SplitBy { iter: self, split, done: None }
    }
}

pub struct SplitBy<'a, I: 'a + Iterator, F> {
    iter: &'a mut I,
    split: F,
    done: Option<Option<I::Item>>,
}

impl<'a, I: 'a + Iterator, F: FnMut(&I::Item) -> bool> Iterator for SplitBy<'a, I, F> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => {
                self.done = Some(None);
                None
            }
            Some(item) => {
                if (self.split)(&item) {
                    self.done = Some(Some(item));
                    None
                } else {
                    Some(item)
                }
            }
        }
    }
}

impl<'a, I: 'a + Iterator, F> SplitBy<'a, I, F> {
    pub fn done(&mut self) -> Option<Option<I::Item>> {
        std::mem::replace(&mut self.done, None)
    }
}
