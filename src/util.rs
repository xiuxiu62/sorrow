use alloc::vec::Vec;

/// Push an item onto a collection, returning the collection
/// 
/// Useful in single line accumulators
pub trait PushReturn<T> {
    fn push_return(&mut self, t: T) -> &mut Self;
}

impl<T> PushReturn<T> for Vec<T> {
    fn push_return(&mut self, t: T) -> &mut Self {
        self.push(t);
        self
    }
} 