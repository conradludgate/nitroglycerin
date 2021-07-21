pub trait Ext: Iterator {
    fn split_by<F>(&mut self, split: F) -> SplitBy<'_, Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool;
}

impl<I: Iterator> Ext for I {
    fn split_by<F>(&mut self, split: F) -> SplitBy<'_, Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool,
    {
        SplitBy { iter: self, split, state: SplitByState::Continue }
    }
}

pub enum SplitByState<T> {
    Continue,
    Split(T),
    Finished
}

pub struct SplitBy<'a, I: 'a + Iterator, F> {
    iter: &'a mut I,
    split: F,
    state: SplitByState<I::Item>,
}

impl<'a, I: 'a + Iterator, F: FnMut(&I::Item) -> bool> Iterator for SplitBy<'a, I, F> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => {
                self.state = SplitByState::Finished;
                None
            }
            Some(item) => {
                if (self.split)(&item) {
                    self.state = SplitByState::Split(item);
                    None
                } else {
                    Some(item)
                }
            }
        }
    }
}

impl<'a, I: 'a + Iterator, F> SplitBy<'a, I, F> {
    pub fn done(&mut self) -> SplitByState<I::Item> {
        std::mem::replace(&mut self.state, SplitByState::Continue)
    }
}
