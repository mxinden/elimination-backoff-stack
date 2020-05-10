mod elimination_array;
mod event;
mod exchanger;
pub mod strategy;
mod treiber_stack;

#[cfg(test)]
mod statistic;

use elimination_array::EliminationArray;
use event::{Event, EventRecorder, NoOpRecorder};
use std::marker::PhantomData;
use strategy::ExpRetryStrategy;
use treiber_stack::TreiberStack;

#[derive(Default)]
pub struct Stack<T, PushS = ExpRetryStrategy, PopS = ExpRetryStrategy> {
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
        self.instrumented_push(item, &mut NoOpRecorder {});
    }

    fn instrumented_push<R: EventRecorder>(&self, item: T, recorder: &mut R) {
        recorder.record(Event::StartPush);

        let mut strategy = PushS::new();

        let mut item = item;

        loop {
            recorder.record(Event::TryStack);
            match self.stack.push(item, &mut strategy) {
                Ok(()) => break,
                Err(i) => item = i,
            };

            if strategy.use_elimination_array() {
                recorder.record(Event::TryEliminationArray);
                match self
                    .elimination_array
                    .exchange_push(item, &mut strategy, recorder)
                {
                    Ok(()) => break,
                    Err(i) => item = i,
                };
            }
        }

        recorder.record(Event::FinishPush);
    }

    pub fn pop(&self) -> Option<T> {
        self.instrumented_pop(&mut NoOpRecorder {})
    }

    fn instrumented_pop<R: EventRecorder>(&self, recorder: &mut R) -> Option<T> {
        recorder.record(Event::StartPop);

        let mut strategy = PopS::new();

        let item = loop {
            recorder.record(Event::TryStack);
            match self.stack.pop(&mut strategy) {
                Ok(item) => break item,
                Err(()) => {}
            };

            if strategy.use_elimination_array() {
                recorder.record(Event::TryEliminationArray);
                match self
                    .elimination_array
                    .exchange_pop(&mut strategy, recorder)
                {
                    Ok(item) => break Some(item),
                    Err(()) => {}
                };
            }
        };

        recorder.record(Event::FinishPop);

        item
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
    use quickcheck::{quickcheck, Arbitrary, Gen, TestResult};
    use rand::Rng;
    use std::convert::TryInto;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use strategy::ExpRetryStrategy;

    // TODO: Say each thread adds monotonically increasing numbers onto the
    // stack. Add a test that ensures that after witnessing an empty stack, one
    // can not see anything lower than one popped off from other threads or
    // pushed onto the stack oneself before.
    //
    // A bit like quiescent consistency.

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

    #[test]
    fn quickcheck_multithreaded_no_duplicates() {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        struct Item {
            thread_id: u8,
            nonce: u32,
        }

        fn prop(num_threads: usize, mut operations: Vec<Vec<Operation<()>>>) -> TestResult {
            if num_threads > num_cpus::get() * 2 || operations.len() < num_threads {
                return TestResult::discard();
            }

            let mut handlers = vec![];
            let stack = Arc::new(Stack::<Item>::new());
            let popped_items = Arc::new(Mutex::new(vec![]));

            // Spawn threads pushing to and popping from stack.
            for thread_id in 0..num_threads {
                let stack = stack.clone();
                let popped_items = popped_items.clone();
                let operations = operations.pop().unwrap();
                if operations.is_empty() {
                    continue;
                }

                handlers.push(thread::spawn(move || {
                    let mut items = vec![];

                    for (nonce, operation) in operations.into_iter().enumerate() {
                        match operation {
                            Operation::Push(_) => {
                                stack.push(Item {
                                    thread_id: thread_id.try_into().unwrap(),
                                    nonce: nonce.try_into().unwrap(),
                                });
                            }
                            Operation::Pop => {
                                if let Some(item) = stack.pop() {
                                    items.push(item);
                                }
                            }
                        };
                    }

                    popped_items.lock().unwrap().extend_from_slice(&items);
                }))
            }

            for handler in handlers {
                handler.join().unwrap();
            }

            let mut popped_items = Arc::try_unwrap(popped_items).unwrap().into_inner().unwrap();
            if popped_items.is_empty() {
                return TestResult::passed();
            }

            // Check for duplicates.
            popped_items.sort();
            let mut prev_item = popped_items.pop().unwrap();
            for item in popped_items {
                if item == prev_item {
                    panic!("Got two equal items: {:?} and {:?}", prev_item, item);
                }

                prev_item = item;
            }

            TestResult::passed()
        }

        quickcheck(prop as fn(_, _) -> _);
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

    #[test]
    fn event_recording() {
        let stack = Arc::new(Stack::<Vec<u8>, ExpRetryStrategy, ExpRetryStrategy>::new());
        // let stack = Arc::new(Stack::<Vec<u8>>::new());
        let item = b"my_test_item".to_vec();
        let item_count = 10_000;

        let mut handlers = vec![];
        let events = Arc::new(Mutex::new(vec![]));

        for _ in 0..(num_cpus::get() / 2) {
            let push_stack = stack.clone();
            let item = item.clone();
            let push_events = events.clone();
            handlers.push(thread::spawn(move || {
                let mut recorder = vec![];
                for _ in 0..item_count {
                    push_stack.instrumented_push(item.clone(), &mut recorder);
                }

                push_events.lock().unwrap().push(recorder);
            }));

            let pop_stack = stack.clone();
            let pop_events = events.clone();
            handlers.push(thread::spawn(move || {
                let mut recorder = vec![];
                for _ in 0..item_count {
                    pop_stack.instrumented_pop(&mut recorder);
                }

                pop_events.lock().unwrap().push(recorder);
            }))
        }

        for handler in handlers {
            handler.join().unwrap();
        }

        let events = Arc::try_unwrap(events).unwrap().into_inner().unwrap();

        statistic::print_report(events.into_iter().flatten().collect());
    }
}
