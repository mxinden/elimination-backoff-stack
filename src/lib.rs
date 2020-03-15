use crossbeam::epoch::{self, Atomic, Owned};
use std::mem::ManuallyDrop;
use std::ptr;
use std::sync::atomic::Ordering::SeqCst;

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

    pub fn exchange_put(&self, item: T) {
        let mut new_item = Owned::new(Item::Waiting(ManuallyDrop::new(item)));

        let guard = epoch::pin();

        loop {
            let current_item = self.item.load(SeqCst, &guard);

            match unsafe { current_item.as_ref() } {
                Some(&Item::Empty) => {
                    match self
                        .item
                        .compare_and_set(current_item, new_item, SeqCst, &guard)
                    {
                        Ok(_) => {
                            unsafe { guard.defer_destroy(current_item) };
                            break;
                        }
                        Err(e) => new_item = e.new,
                    }
                }
                Some(&Item::Waiting(_)) => {
                    unimplemented!();
                }
                Some(&Item::Busy) => {
                    unimplemented!();
                }
                None => unimplemented!(),
            }
        }

        loop {
            let current_item = self.item.load(SeqCst, &guard);

            match unsafe { current_item.as_ref() } {
                Some(&Item::Empty) => {
                    panic!("only we can set it back to empty");
                }
                Some(&Item::Waiting(_)) => {
                    continue;
                }
                Some(&Item::Busy) => {
                    self.item
                        .compare_and_set(current_item, Owned::new(Item::Empty), SeqCst, &guard)
                        .expect("we should be the only one compare and swapping this value");
                    unsafe { guard.defer_destroy(current_item) };
                    return;
                }
                None => unimplemented!(),
            }
        }
    }

    pub fn exchange_pop(&self) -> T {
        let guard = epoch::pin();

        loop {
            let current_item = self.item.load(SeqCst, &guard);

            match unsafe { current_item.as_ref() } {
                Some(&Item::Empty) => {
                    continue;
                }
                Some(&Item::Waiting(ref item)) => {
                    if self
                        .item
                        .compare_and_set(current_item, Owned::new(Item::Busy), SeqCst, &guard)
                        .is_ok()
                    {
                        unsafe {
                            guard.defer_destroy(current_item);
                            return ManuallyDrop::into_inner(ptr::read(&(*item)));
                        }
                    }
                }
                Some(&Item::Busy) => {
                    unimplemented!();
                }
                None => unimplemented!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn put_pop() {
        let exchanger = Arc::new(Exchanger::new());

        let t1_exchanger = exchanger.clone();
        let t1 = thread::spawn(move || {
            t1_exchanger.exchange_put(());
        });

        assert_eq!(exchanger.exchange_pop(), ());

        t1.join().unwrap();
    }
}
