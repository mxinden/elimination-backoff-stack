use crate::exchanger::Exchanger;
use rand::{thread_rng, Rng};

#[derive(Default)]
pub struct EliminationArray<T> {
    exchangers: Vec<Exchanger<T>>,
}

impl<T> EliminationArray<T> {
    pub fn new() -> Self {
        // TODO: Is num_cpus or num_cpus / 2 the better init? The latter would
        // cause more heterogeneous as well as homogeneous collisions. The
        // former being good, the latter bad.
        let exchangers = (0..num_cpus::get()).map(|_| Exchanger::new()).collect();

        Self { exchangers }
    }

    pub fn exchange_put(&self, item: T) -> Result<(), T> {
        self.rnd_exchanger().exchange_put(item)
    }

    pub fn exchange_pop(&self) -> Result<T, ()> {
        self.rnd_exchanger().exchange_pop()
    }

    fn rnd_exchanger(&self) -> &Exchanger<T> {
        let i = thread_rng().gen_range(0, self.exchangers.len());
        &self.exchangers[i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn put_pop_num_cpus() {
        let item_count = 10_000;

        let mut handlers = vec![];
        let elimination_array = Arc::new(EliminationArray::new());

        // Put threads.
        for _ in 0..(num_cpus::get() / 2) {
            let elimination_array = elimination_array.clone();

            handlers.push(thread::spawn(move || {
                for _ in 0..item_count {
                    while elimination_array.exchange_put(()).is_err() {}
                }
            }))
        }

        // Pop threads.
        for _ in 0..(num_cpus::get() / 2) {
            let elimination_array = elimination_array.clone();

            handlers.push(thread::spawn(move || {
                for _ in 0..item_count {
                    while elimination_array.exchange_pop().is_err() {}
                }
            }))
        }

        for handler in handlers {
            handler.join().unwrap();
        }
    }
}
