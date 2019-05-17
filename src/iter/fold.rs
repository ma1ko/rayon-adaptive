//! Fold and avoid local reductions.
use crate::prelude::*;
use crate::Policy;
use derive_divisible::{Divisible, IntoIterator};
use std::marker::PhantomData;
use std::option::IntoIter;

/// The `Fold` struct is a parallel folder, returned by the `fold` method on `ParallelIterator`.
/// It is for use when the reduction operation comes with overhead.
/// So instead of reducing all tiny pieces created by local iterators we just
/// reduce for the real divisions.
#[derive(Divisible, IntoIterator)]
#[power(P)]
#[item(O)]
pub struct Fold<
    P: Power,
    I: ParallelIterator<P>,
    O: Send,
    ID: Fn() -> O + Clone + Send,
    F: Fn(O, I::Item) -> O + Clone + Send,
> {
    pub(crate) remaining_input: I,
    #[divide_by(default)]
    pub(crate) current_output: Option<O>,
    #[divide_by(clone)]
    pub(crate) identity: ID,
    #[divide_by(clone)]
    pub(crate) fold_op: F,
    #[divide_by(default)]
    pub(crate) phantom: PhantomData<P>,
}

impl<
        P: Power,
        I: ParallelIterator<P>,
        O: Send,
        ID: Fn() -> O + Clone + Send,
        F: Fn(O, I::Item) -> O + Clone + Send,
    > ParallelIterator<P> for Fold<P, I, O, ID, F>
{
    type Item = O;
    type SequentialIterator = IntoIter<O>;
    fn iter(mut self, size: usize) -> (Self::SequentialIterator, Self) {
        let final_call = self.base_length().expect("cannot fold infinite sizes") == size;
        let (sequential_iterator, new_remaining_input) = self.remaining_input.iter(size);
        let current_output = self.current_output.take().unwrap_or_else(&self.identity);
        let new_output = sequential_iterator.fold(current_output, &self.fold_op);
        self.remaining_input = new_remaining_input;
        (
            if final_call {
                Some(new_output)
            } else {
                self.current_output = Some(new_output); // we put it back here
                None
            }
            .into_iter(),
            self,
        )
    }
    fn policy(&self) -> Policy {
        self.remaining_input.policy()
    }
    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        self.remaining_input.blocks_sizes()
    }
}