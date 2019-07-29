use crate::prelude::*;

/// A peekable iterator, allowing to peek at a specified index in the underlying data
pub trait PeekableIterator: IndexedParallelIterator {
    /// Peeks into the iterator without consuming it, returning the item at the specified location
    fn peek(&self, index: usize) -> Self::Item;
}
