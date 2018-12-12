//! This module contains all traits enabling us to express some parallelism.
use std;
use std::ops::Range;
use std::ptr;

use chunks::Chunks;
// pub use iter::AdaptiveFolder;
use policy::ParametrizedInput;
use Policy;

pub trait Divisible: Sized + Send + Sync {
    /// Divide ourselves.
    fn divide(self) -> (Self, Self);
    /// Return our length.
    fn base_length(&self) -> usize;
    /// Is there something left to do ?
    fn is_empty(&self) -> bool {
        self.base_length() == 0
    }
    fn with_policy(self, policy: Policy) -> ParametrizedInput<Self> {
        ParametrizedInput {
            input: self,
            policy,
        }
    }
}

pub trait DivisibleIntoBlocks: Divisible {
    /// Divide ourselves where requested.
    fn divide_at(self, index: usize) -> (Self, Self);
    /// Divide ourselves keeping right part in self.
    /// Returns the left part.
    /// NB: this is useful for iterators creation.
    fn cut_left_at(&mut self, index: usize) -> Self {
        // there is a lot of unsafe going on here.
        // I think it's ok. rust uses the same trick for moving iterators (vecs for example)
        unsafe {
            let my_copy = ptr::read(self);
            let (left, right) = my_copy.divide_at(index);
            let pointer_to_self = self as *mut Self;
            ptr::write(pointer_to_self, right);
            left
        }
    }
    /// Get a sequential iterator on chunks of Self of given sizes.
    fn chunks<S: Iterator<Item = usize>>(self, sizes: S) -> Chunks<Self, S> {
        Chunks {
            remaining: self,
            remaining_sizes: sizes,
        }
    }
}

pub trait DivisibleAtIndex: DivisibleIntoBlocks {}

impl<'a, T: Sync> Divisible for &'a [T] {
    fn base_length(&self) -> usize {
        (*self as &[T]).len()
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.len() / 2;
        self.split_at(mid)
    }
}

impl<'a, T: Sync> DivisibleIntoBlocks for &'a [T] {
    fn divide_at(self, index: usize) -> (Self, Self) {
        self.split_at(index)
    }
}

impl<'a, T: Sync> DivisibleAtIndex for &'a [T] {}

//TODO: I don't get why the compiler requires send here
impl<'a, T: 'a + Sync + Send> Divisible for &'a mut [T] {
    fn base_length(&self) -> usize {
        (*self as &[T]).len()
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.base_length() / 2;
        self.split_at_mut(mid)
    }
}

impl<'a, T: 'a + Sync + Send> DivisibleIntoBlocks for &'a mut [T] {
    fn divide_at(self, index: usize) -> (Self, Self) {
        self.split_at_mut(index)
    }
}

impl<'a, T: 'a + Sync + Send> DivisibleAtIndex for &'a mut [T] {}

//TODO: be more generic but it seems complex
impl Divisible for Range<usize> {
    fn base_length(&self) -> usize {
        ExactSizeIterator::len(self)
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.start + ExactSizeIterator::len(&self) / 2;
        (self.start..mid, mid..self.end)
    }
}

//TODO: be more generic but it seems complex
impl DivisibleIntoBlocks for Range<usize> {
    fn divide_at(self, index: usize) -> (Self, Self) {
        (
            self.start..(self.start + index),
            (self.start + index)..self.end,
        )
    }
}

impl DivisibleAtIndex for Range<usize> {}

// //TODO: macroize all that stuff ; even better : derive ?
// impl<A: Divisible, B: Divisible> Divisible for (A, B) {
//     fn base_length(&self) -> usize {
//         std::cmp::min(self.0.base_length(), self.1.base_length())
//     }
//     fn divide(self) -> (Self, Self) {
//         let (left_a, right_a) = self.0.divide();
//         let (left_b, right_b) = self.1.divide();
//         ((left_a, left_b), (right_a, right_b))
//     }
// }
//
// //TODO: macroize all that stuff ; even better : derive ?
// impl<A: DivisibleIntoBlocks, B: DivisibleIntoBlocks> DivisibleIntoBlocks for (A, B) {
//     fn divide_at(self, index: usize) -> (Self, Self) {
//         let (left_a, right_a) = self.0.divide_at(index);
//         let (left_b, right_b) = self.1.divide_at(index);
//         ((left_a, left_b), (right_a, right_b))
//     }
// }
//
// impl<A: DivisibleAtIndex, B: DivisibleAtIndex> DivisibleAtIndex for (A, B) {}

impl<A: Divisible, B: Divisible, C: Divisible> Divisible for (A, B, C) {
    fn base_length(&self) -> usize {
        std::cmp::min(
            self.0.base_length(),
            std::cmp::min(self.1.base_length(), self.2.base_length()),
        )
    }
    fn divide(self) -> (Self, Self) {
        let (left_a, right_a) = self.0.divide();
        let (left_b, right_b) = self.1.divide();
        let (left_c, right_c) = self.2.divide();
        ((left_a, left_b, left_c), (right_a, right_b, right_c))
    }
}
