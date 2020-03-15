// Adapted from https://github.com/crossbeam-rs/crossbeam/blob/master/crossbeam-epoch/examples/treiber_stack.rs

use crossbeam::epoch;

use std::mem::ManuallyDrop;
use std::ptr;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

use epoch::{Atomic, Owned};

/// Treiber's lock-free stack.
///
/// Usable with any number of producers and consumers.
#[derive(Debug, Default)]
pub struct TreiberStack<T> {
    head: Atomic<Node<T>>,
}

#[derive(Debug)]
struct Node<T> {
    data: ManuallyDrop<T>,
    next: Atomic<Node<T>>,
}

impl<T> TreiberStack<T> {
    /// Creates a new, empty stack.
    pub fn new() -> TreiberStack<T> {
        TreiberStack {
            head: Atomic::null(),
        }
    }

    /// Pushes a value on top of the stack.
    pub fn push(&self, t: T) -> Result<(), T> {
        let n = Owned::new(Node {
            data: ManuallyDrop::new(t),
            next: Atomic::null(),
        });

        let guard = epoch::pin();

        let head = self.head.load(Relaxed, &guard);
        n.next.store(head, Relaxed);

        match self.head.compare_and_set(head, n, Release, &guard) {
            Ok(_) => Ok(()),
            Err(e) => {
                // TODO: Rust's Box supports DerefMove which returns an owned T
                // on dereferencing the Box. This must be possible with Owned as
                // well somehow. Creating a Box first most involve some
                // overhead.
                //
                // See:
                // https://stackoverflow.com/questions/42264041/how-do-i-get-an-owned-value-out-of-a-box
                Err(ManuallyDrop::into_inner((*e.new.into_box()).data))
            },
        }
    }

    /// Attempts to pop the top element from the stack.
    pub fn pop(&self) -> Result<Option<T>, ()> {
        let guard = epoch::pin();
        let head = self.head.load(Acquire, &guard);

        match unsafe { head.as_ref() } {
            Some(h) => {
                let next = h.next.load(Relaxed, &guard);

                match self
                    .head
                    .compare_and_set(head, next, Release, &guard)
                {
                    Ok(_) => unsafe {
                        guard.defer_destroy(head);
                        Ok(Some(ManuallyDrop::into_inner(ptr::read(&(*h).data))))
                    },
                    Err(_) => Err(())
                }
            }
            None => Ok(None),
        }
    }
}

impl<T> Drop for TreiberStack<T> {
    fn drop(&mut self) {
        // TODO: Document unwrap.
        while self.pop().unwrap().is_some() {}
    }
}
