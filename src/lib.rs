mod elimination_array;
mod exchanger;
pub mod strategy;
mod treiber_stack;

use elimination_array::EliminationArray;
use std::marker::PhantomData;
use strategy::DefaultStrategy;
use treiber_stack::TreiberStack;

#[derive(Default)]
pub struct Stack<T, PushS = DefaultStrategy, PopS = DefaultStrategy> {
    stack: TreiberStack<T>,
    elimination_array: EliminationArray<T>,
    phantom: PhantomData<(PushS, PopS)>,
}

impl<T, PushS, PopS> Stack<T, PushS, PopS>
where
    PushS: PushStrategy,
    PopS: PopStrategy,
{
    pub fn new() -> Self {
        Self {
            stack: TreiberStack::new(),
            elimination_array: EliminationArray::new(),
            phantom: PhantomData,
        }
    }

    pub fn push(&self, item: T) {
        let mut strategy = PushS::new();

        let mut item = item;

        loop {
            match self.stack.push(item, &mut strategy) {
                Ok(()) => return,
                Err(i) => item = i,
            };

            if strategy.use_elimination_array() {
                match self.elimination_array.exchange_push(item, &mut strategy) {
                    Ok(()) => return,
                    Err(i) => item = i,
                };
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        let mut strategy = PopS::new();

        loop {
            match self.stack.pop(&mut strategy) {
                Ok(item) => return item,
                Err(()) => {}
            };

            if strategy.use_elimination_array() {
                match self.elimination_array.exchange_pop(&mut strategy) {
                    Ok(item) => return Some(item),
                    Err(()) => {}
                };
            }
        }
    }
}

/// Strategy for push operations.
pub trait PushStrategy: treiber_stack::PushStrategy + elimination_array::PushStrategy {
    fn new() -> Self;

    /// Decide whether the stack should try eliminating the push operation on
    /// the elimination array next. Is called each time such elimination is
    /// possible.
    fn use_elimination_array(&mut self) -> bool;
}

/// Strategy for pop operations.
pub trait PopStrategy: treiber_stack::PopStrategy + elimination_array::PopStrategy {
    fn new() -> Self;

    /// Decide whether the stack should try eliminating the pop operation on the
    /// elimination array next. Is called each time such elimination is
    /// possible.
    fn use_elimination_array(&mut self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{quickcheck, Arbitrary, Gen};
    use rand::Rng;
    use std::sync::Arc;
    use std::thread;

    #[derive(Clone, Debug)]
    enum Operation<T> {
        Push(T),
        Pop,
    }

    impl<T: Arbitrary> Arbitrary for Operation<T> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            if g.gen::<bool>() {
                Operation::Push(Arbitrary::arbitrary(g))
            } else {
                Operation::Pop
            }
        }
    }

    #[test]
    fn quickcheck_single_threaded_compare_to_vec() {
        fn prop(operations: Vec<Operation<usize>>) {
            let elimination_backoff_stack: Stack<usize> = Stack::new();
            let mut vec_stack: Vec<usize> = vec![];

            for operation in operations {
                match operation {
                    Operation::Push(item) => {
                        elimination_backoff_stack.push(item.clone());
                        vec_stack.push(item);
                    }
                    Operation::Pop => assert_eq!(elimination_backoff_stack.pop(), vec_stack.pop()),
                }
            }
        }

        quickcheck(prop as fn(_));
    }

    /// Scenario: A push or pop operation fails on the lock-free stack due to
    /// contention on the `head` pointer and thus eludes to the elimination
    /// array. In case contention is gone instantly all opposite operations will
    /// hit the lock-free stack directly. Thereby the push or pop operation
    /// starves on the elimination array.
    ///
    /// Ensure that push or pop operation don't starve, e.g. by falling back to
    /// trying the lock-free stack from the elimination array.
    ///
    /// Tested here by spawning only threads that push or pop to the stack.
    /// Probability for contention is high, thus resulting in some starved
    /// threads on the elimination array.
    #[test]
    fn ensure_push_or_pop_does_not_starve_on_array() {
        enum Operation {
            Push,
            Pop,
        }

        for operation in [Operation::Push, Operation::Pop].iter() {
            let item_count = 100_000;

            let mut handlers = vec![];
            let stack = Arc::new(Stack::<()>::new());

            // When we test `pop` push some values onto stack beforehand to make
            // `pop` operation more involved, thus cause more contention further
            // below.
            if let Operation::Pop = operation {
                for _ in 0..item_count {
                    stack.push(());
                }
            }

            for _ in 0..num_cpus::get() {
                let stack = stack.clone();

                handlers.push(thread::spawn(move || {
                    for _ in 0..item_count {
                        match operation {
                            Operation::Push => {
                                stack.push(());
                            }
                            Operation::Pop => {
                                stack.pop();
                            }
                        };
                    }
                }))
            }

            for handler in handlers {
                handler.join().unwrap();
            }
        }
    }
}
