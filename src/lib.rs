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
            Ok(()) => return,
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
