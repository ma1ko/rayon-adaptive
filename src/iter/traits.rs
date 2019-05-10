//! Iterator governing traits.
//! `Edible` allows for a step by step extraction of sequential work from parallel iterator.
use super::ByBlocks;
use super::IteratorFold;
use super::WithPolicy;
use crate::divisibility::{BasicPower, BlockedPower, IndexedPower};
use crate::prelude::*;
use crate::schedulers::schedule;
use crate::Policy;
use std::cmp::max;
use std::iter::{empty, once};
use std::marker::PhantomData;

/// We can produce sequential iterators to be eaten slowly.
pub trait Edible: Sized + Send {
    /// This registers the type of output produced (it IS the item of the SequentialIterator).
    type Item: Send; // TODO: can we get rid of that and keep a short name ?
    /// This registers the type of iterators produced.
    type SequentialIterator: Iterator<Item = Self::Item>;
    /// Give us a sequential iterator corresponding to `size` iterations.
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self);
    /// Return current scheduling `Policy`.
    fn policy(&self) -> Policy {
        Policy::Rayon
    }
}

/// This traits enables to implement all basic methods for all type of iterators.
pub trait ParallelIterator<P: Power>: Divisible<P> + Edible {
    /// Return an iterator on sizes of all macro blocks.
    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        Box::new(empty())
    }
    /// Fold each sequential iterator into a single value.
    /// See the max method below as a use case.
    fn iterator_fold<R, F>(self, fold_op: F) -> IteratorFold<R, P, Self, F>
    where
        R: Sized + Send,
        F: Fn(Self::SequentialIterator) -> R + Send + Clone,
    {
        IteratorFold {
            iterator: self,
            fold: fold_op,
            phantom: PhantomData,
        }
    }
    /// Sets scheduling policy.
    fn with_policy(self, policy: Policy) -> WithPolicy<P, Self> {
        WithPolicy {
            policy,
            iterator: self,
            phantom: PhantomData,
        }
    }
    /// Sets the macro-blocks sizes.
    fn by_blocks<I: Iterator<Item = usize> + Send + 'static>(self, sizes: I) -> ByBlocks<P, Self> {
        ByBlocks {
            sizes_iterator: Some(Box::new(sizes)),
            iterator: self,
            phantom: PhantomData,
        }
    }
    /// Reduce with call to scheduler.
    fn reduce<OP, ID>(mut self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        let policy = self.policy();
        let sizes = self.blocks_sizes();
        schedule(policy, &mut self.blocks(sizes), &identity, &op)
    }
    /// Return the max of all elements.
    ///
    /// # Example
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!((0u32..100).max(), Some(99))
    /// ```
    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        self.iterator_fold(Iterator::max).reduce(|| None, max)
    }
}

/// Here go all methods for basic power only.
pub trait BasicParallelIterator: ParallelIterator<BasicPower> {
    /// slow find
    fn find(self) {
        unimplemented!()
    }
}

//TODO: WE NEED A METHOD FOR COLLECT UP TO BLOCKED

/// Here go all methods for blocked or more.
pub trait BlockedParallelIterator: ParallelIterator<BlockedPower> {
    /// fast find
    fn find(self) {
        unimplemented!()
    }
}

/// Here go all methods for indexed.
pub trait IndexedParallelIterator: ParallelIterator<IndexedPower> {
    /// zip two iterators
    fn zip() {
        unimplemented!()
    }
}