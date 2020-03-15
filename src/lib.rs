mod exchanger;
mod elimination_array;
mod treiber_stack;

use elimination_array::EliminationArray;
use treiber_stack::TreiberStack;

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
        match self.stack.push(item) {
            Ok(()) => {},
            // TODO: What if there is contention thus the thread tries the
            // array, but the contention resolves, thus all pop operations go to
            // the stack, find nothing and return `None`.
            Err(item) => self.elimination_array.exchange_put(item),
        }
    }

    pub fn pop(&self) -> Option<T> {
        match self.stack.pop() {
            Ok(item) => item,
            Err(()) => Some(self.elimination_array.exchange_pop())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use quickcheck::{Arbitrary, Gen, quickcheck};

    #[derive(Clone, Debug)]
    enum Operation<T> {
        Push(T),
        Pop
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
                    },
                    Operation::Pop => {
                        assert_eq!(elimination_backoff_stack.pop(), vec_stack.pop())
                    }
                }
            }
        }

        quickcheck(prop as fn(_));
    }
}
