use crossbeam::epoch::{self, Atomic, Owned};
use std::mem::ManuallyDrop;
use std::ptr;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

// TODO: crossbeam::epoch::Shared has a with_tag method. Can this mirror the
// Java AtomicStampedReference?
enum Item<T> {
    Empty,
    // TODO: ManuallyDrop necessary here?
    Waiting(ManuallyDrop<T>),
    Busy,
}

pub struct Exchanger<T> {
    item: Atomic<Item<T>>,
}

impl<T> Exchanger<T> {
    pub fn new() -> Self {
        Self {
            item: Atomic::new(Item::Empty),
        }
    }

    pub fn exchange_push<S: PushStrategy>(&self, item: T, strategy: &mut S) -> Result<(), T> {
        let mut new_item = Owned::new(Item::Waiting(ManuallyDrop::new(item)));

        // TODO: Should we reuse this guard? Might be better performing when
        // calling `exchange_push` in a loop.
        let guard = epoch::pin();

        loop {
            if !strategy.try_start_exchange() {
                let item = match std::mem::replace(&mut *new_item, Item::Empty) {
                    Item::Empty => unreachable!(),
                    Item::Waiting(item) => ManuallyDrop::into_inner(item),
                    Item::Busy => unreachable!(),
                };

                return Err(item);
            }

            // Assume using `Relaxed` is correct, given that the actual
            // synchronization happens further below with `compare_and_set`.
            let current_item = self.item.load(Relaxed, &guard);

            match unsafe { current_item.as_ref() } {
                Some(&Item::Empty) => {
                    match self
                        .item
                        // Assume using `Release` is correct here, given that
                        // one needs to enforce that `new_item` is written
                        // before being accessible by other threads through this
                        // `compare_and_set`.
                        .compare_and_set(current_item, new_item, Release, &guard)
                    {
                        Ok(_) => {
                            unsafe { guard.defer_destroy(current_item) };
                            break;
                        }
                        Err(e) => new_item = e.new,
                    }
                }
                Some(&Item::Waiting(_)) => continue,
                Some(&Item::Busy) => continue,
                None => unimplemented!(),
            }
        }

        loop {
            // TODO: Don't yield to the OS, but busy loop for a bit (can happen
            // in the strategy).
            //
            // See parking_lot spinwait.rs
            std::thread::yield_now();

            // Assume using `Relaxed` is correct, given that the actual
            // synchronization happens further below with `compare_and_set`.
            let current_item = self.item.load(Relaxed, &guard);

            match unsafe { current_item.as_ref() } {
                Some(&Item::Empty) => {
                    panic!("only we can set it back to empty");
                }
                Some(&Item::Waiting(ref item)) => {
                    if strategy.retry_check_exchanged() {
                        continue;
                    }

                    if self
                        .item
                        // Assume using `Release` is correct, given that
                        // correctness depends on the fact that the previous
                        // `compare_and_set` going from `Empty` to `Waiting`
                        // happens before this instruction. Otherwise nothing
                        // enforces, that the `Exchanger` was filled by this
                        // push operation and not by a different push operation.
                        .compare_and_set(current_item, Owned::new(Item::Empty), Release, &guard)
                        .is_ok()
                    {
                        unsafe {
                            guard.defer_destroy(current_item);
                            return Err(ManuallyDrop::into_inner(ptr::read(&(*item))));
                        }
                    }
                }
                Some(&Item::Busy) => {
                    self.item
                        // Assume using `Release` is correct, given that
                        // correctness depends on the fact that the previous
                        // `compare_and_set` going from `Empty` to `Waiting`
                        // happens before this instruction. Otherwise nothing
                        // enforces, that the `Exchanger` was filled by this
                        // push operation and not by a different push operation.
                        .compare_and_set(current_item, Owned::new(Item::Empty), Release, &guard)
                        .expect("we should be the only one compare and swapping this value");
                    unsafe { guard.defer_destroy(current_item) };
                    return Ok(());
                }
                None => unimplemented!(),
            }
        }
    }

    pub fn exchange_pop<S: PopStrategy>(&self, strategy: &mut S) -> Result<T, ()> {
        let guard = epoch::pin();

        while strategy.try_exchange() {
            // Assume using `Relaxed` is correct, given that the actual
            // synchronization happens further below with `compare_and_set`.
            let current_item = self.item.load(Relaxed, &guard);

            match unsafe { current_item.as_ref() } {
                Some(&Item::Empty) => {
                    strategy.on_no_contention();
                    continue;
                }
                Some(&Item::Waiting(ref item)) => {
                    match self
                        .item
                        // Assume using `Acquire` is correct, given that this
                        // operation does not depend on any previous operations
                        // happening before, but past operations (returning the
                        // item) happening after.
                        .compare_and_set(current_item, Owned::new(Item::Busy), Acquire, &guard)
                    {
                        Ok(_) => {
                            unsafe {
                                guard.defer_destroy(current_item);
                                return Ok(ManuallyDrop::into_inner(ptr::read(&(*item))));
                            }
                        },
                        Err(_) => strategy.on_contention(),
                    }
                }
                Some(&Item::Busy) => {
                    strategy.on_contention();
                    continue;
                }
                None => unimplemented!(),
            }
        }

        Err(())
    }
}

// TODO: Rethink this implementation. What about the ManuallyDrop wrapping Item?
impl<T> Drop for Exchanger<T> {
    fn drop(&mut self) {
        let owned: Owned<_>;
        unsafe {
            // By now the DataStructure lives only in our thread and we are sure we
            // don't hold any Shared or & to it ourselves.
            owned = std::mem::replace(&mut self.item, Atomic::null()).into_owned();
        }

        let boxed: Box<_> = owned.into_box();
        let mut item: Item<_> = *boxed;

        // Make sure to access `Item<_>` and not `ManuallyDrop<Item<_>>`.
        match item {
            Item::Empty => {}
            Item::Busy => {}
            Item::Waiting(ref mut item) => {
                unsafe { ManuallyDrop::drop(item) };
            }
        }

        drop(item);
    }
}

pub trait PushStrategy {
    fn try_start_exchange(&mut self) -> bool;
    fn retry_check_exchanged(&mut self) -> bool;
}

pub trait PopStrategy {
    fn try_exchange(&mut self) -> bool;

    fn on_contention(&mut self) {}
    fn on_no_contention(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::DefaultStrategy;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn push_pop_2_threads() {
        let exchanger = Arc::new(Exchanger::new());

        let t1_exchanger = exchanger.clone();
        let mut push_strategy = DefaultStrategy::new();
        let t1 =
            thread::spawn(
                move || while t1_exchanger.exchange_push((), &mut push_strategy).is_err() {},
            );

        let mut pop_strategy = DefaultStrategy::new();
        while exchanger.exchange_pop(&mut pop_strategy).is_err() {}

        t1.join().unwrap();
    }

    #[test]
    fn push_pop_4_threads() {
        let mut handlers = vec![];
        let exchanger = Arc::new(Exchanger::new());

        let t1_exchanger = exchanger.clone();
        let mut t1_strategy = DefaultStrategy::new();
        handlers.push(thread::spawn(move || {
            while t1_exchanger.exchange_push((), &mut t1_strategy).is_err() {}
        }));

        let t2_exchanger = exchanger.clone();
        let mut t2_strategy = DefaultStrategy::new();
        handlers.push(thread::spawn(move || {
            while t2_exchanger.exchange_push((), &mut t2_strategy).is_err() {}
        }));

        let t3_exchanger = exchanger.clone();
        let mut t3_strategy = DefaultStrategy::new();
        handlers.push(thread::spawn(move || {
            while t3_exchanger.exchange_pop(&mut t3_strategy).is_err() {}
        }));

        let mut t4_strategy = DefaultStrategy::new();
        while exchanger.exchange_pop(&mut t4_strategy).is_err() {}

        for handler in handlers.into_iter() {
            handler.join().unwrap();
        }
    }
}
