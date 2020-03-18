mod elimination_array;
mod exchanger;
mod treiber_stack;

use elimination_array::EliminationArray;
use treiber_stack::TreiberStack;

#[derive(Default)]
pub struct Stack<T> {
    stack: TreiberStack<T>,
    elimination_array: EliminationArray<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self {
            stack: TreiberStack::new(),
            elimination_array: EliminationArray::new(),
        }
    }

    // TODO: Be consistent across the crate. Either `put` or `push`.
    pub fn push(&self, item: T) {
        let mut item = item;
        loop {
            match self.stack.push(item) {
                Ok(()) => return,
                Err(i) => item = i,
            };

            match self.elimination_array.exchange_put(item) {
                Ok(()) => return,
                Err(i) => item = i,
            };
        }
    }

    pub fn pop(&self) -> Option<T> {
        loop {
            match self.stack.pop() {
                Ok(item) => return item,
                Err(()) => {}
            };

            match self.elimination_array.exchange_pop() {
                Ok(item) => return Some(item),
                Err(()) => {}
            };
        }
    }
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

    // TODO: What about pop? Does it starve?
    /// Scenario: A push operation fails on the lock-free stack due to
    /// contention on the `head` pointer and thus eludes to the elimination
    /// array. In case contention is gone instantly all pop operations will hit
    /// the lock-free stack directly. Thereby the push operation starves.
    ///
    /// Ensure that the push operation doesn't starve, e.g. by falling back to
    /// trying the lock-free stack from the elimination array.
    ///
    /// Tested here by spawning only threads that push to the stack. Probability
    /// for contention is high, thus resulting in some starved push threads in
    /// the elimination array.
    #[test]
    fn ensure_put_does_not_starve_on_array() {
        let item_count = 10_000;

        let mut handlers = vec![];
        let stack = Arc::new(Stack::new());

        // Push threads.
        for _ in 0..num_cpus::get() {
            let stack = stack.clone();

            handlers.push(thread::spawn(move || {
                for _ in 0..item_count {
                    stack.push(());
                }
            }))
        }

        for handler in handlers {
            handler.join().unwrap();
        }
    }
}
