use crate::prelude::*;
use crate::small_channel::small_channel;

/// This just does one block.
/// It is a new version, pretty nifty as it fuses all schedulers into one.
/// It also allows us to avoid policies since all policies are just iterator adaptors now.
pub(crate) fn schedule_reduce<I, ID, OP>(
    iterator: I,
    identity: &ID,
    op: &OP,
    output: I::Item,
) -> I::Item
where
    I: FiniteParallelIterator + Divisible,
    OP: Fn(I::Item, I::Item) -> I::Item + Sync,
    ID: Fn() -> I::Item + Sync,
{
    // for now just a non adaptive version
    if iterator.is_divisible() {
        let (left, right) = iterator.divide();
        let (left_answer, right_answer) = rayon::join(
            || schedule_reduce(left, identity, op, output),
            || schedule_reduce(right, identity, op, identity()),
        );
        op(left_answer, right_answer)
    } else {
        schedule_adaptive(iterator, identity, op, output)
    }
}

pub(crate) fn schedule_adaptive<I, ID, OP>(
    iterator: I,
    identity: &ID,
    op: &OP,
    output: I::Item,
) -> I::Item
where
    I: FiniteParallelIterator + Divisible,
    OP: Fn(I::Item, I::Item) -> I::Item + Sync,
    ID: Fn() -> I::Item + Sync,
{
    let (sender, receiver) = small_channel();
    let (left_result, maybe_right_result): (I::Item, Option<I::Item>) = rayon::join_context(
        |_| match iterator
            .micro_blocks_sizes()
            .take_while(|_| !sender.receiver_is_waiting())
            .try_fold((iterator, output), |(mut iterator, output), s| {
                let size = std::cmp::min(s, iterator.len());
                let new_output = {
                    let sequential_iterator = iterator.sequential_borrow_on_left_for(size);
                    sequential_iterator.fold(output, op)
                };
                if iterator.len() == 0 {
                    // it's over
                    Err(new_output)
                } else {
                    // something is left
                    Ok((iterator, new_output))
                }
            }) {
            Ok((remaining_iterator, output)) => {
                // we are being stolen. Let's give something.
                let (my_half, his_half) = remaining_iterator.divide();
                sender.send(Some(his_half));
                schedule_reduce(my_half, identity, op, output)
            }
            Err(output) => {
                // all is completed, cancel stealer's task.
                sender.send(None);
                output
            }
        },
        |c| {
            if c.migrated() {
                receiver
                    .recv()
                    .expect("receiving adaptive iterator failed")
                    .map(|iterator| schedule_reduce(iterator, identity, op, identity()))
            } else {
                None
            }
        },
    );
    if let Some(right_result) = maybe_right_result {
        op(left_result, right_result)
    } else {
        left_result
    }
}