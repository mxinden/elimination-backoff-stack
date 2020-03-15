use crate::exchanger::Exchanger;
use rand::{thread_rng, Rng};

pub struct EliminationArray<T> {
    exchangers: Vec<Exchanger<T>>,
}

impl<T> EliminationArray<T> {
    pub fn new(capacity: usize) -> Self {
        let exchangers = (0..capacity).map(|_| Exchanger::new()).collect();

        Self { exchangers }
    }

    pub fn exchange_put(&self, item: T) {
        let mut item = item;

        loop {
            match self.rnd_exchanger().exchange_put(item) {
                Ok(()) => return,
                Err(i) => item = i,
            }
        }
    }

    pub fn exchange_pop(&self) -> T {
        loop {
            match self.rnd_exchanger().exchange_pop() {
                Ok(item) => return item,
                Err(()) => continue,
            }
        }
    }

    fn rnd_exchanger(&self) -> &Exchanger<T> {
        let i = thread_rng().gen_range(0, self.exchangers.len());
        &self.exchangers[i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn put_pop_num_cpus() {
        let item_count = 10_000;

        let barrier = Arc::new(Barrier::new(num_cpus::get() / 2 + 1));
        let mut handlers = vec![];
        let elimination_array = Arc::new(EliminationArray::new(num_cpus::get()));

        // Put threads.
        for _ in 0..(num_cpus::get() / 2) {
            let barrier = barrier.clone();
            let elimination_array = elimination_array.clone();

            handlers.push(thread::spawn(move || {
                for _ in 0..item_count {
                    elimination_array.exchange_put(());
                }

                barrier.wait();
            }))
        }

        // Pop threads.
        for _ in 0..(num_cpus::get() / 2) {
            let elimination_array = elimination_array.clone();

            handlers.push(thread::spawn(move || loop {
                assert_eq!(elimination_array.exchange_pop(), ());
            }))
        }

        barrier.wait();
    }
}
