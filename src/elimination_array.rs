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

    pub fn exchange_push(&self, item: T) -> Result<(), T> {
        self.rnd_exchanger().exchange_push(item)
    }

    pub fn exchange_pop<S: PopStrategy>(&self, strategy: &mut S) -> Result<T, ()> {
        while strategy.try_pop() {
            match self.rnd_exchanger().exchange_pop() {
                Ok(item) => return Ok(item),
                Err(()) => {}
            }
        }

        return Err(());
    }

    fn rnd_exchanger(&self) -> &Exchanger<T> {
        let i = thread_rng().gen_range(0, self.exchangers.len());
        &self.exchangers[i]
    }
}

pub trait PopStrategy {
    fn try_pop(&mut self) -> bool;
}

#[cfg(test)]
mod tests {
    use crate::strategy::DefaultStrategy;
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn push_pop_num_cpus() {
        let item_count = 10_000;

        let mut handlers = vec![];
        let elimination_array = Arc::new(EliminationArray::new());

        // Push threads.
        for _ in 0..(num_cpus::get() / 2) {
            let elimination_array = elimination_array.clone();

            handlers.push(thread::spawn(move || {
                for _ in 0..item_count {
                    while elimination_array.exchange_push(()).is_err() {}
                }
            }))
        }

        // Pop threads.
        for _ in 0..(num_cpus::get() / 2) {
            let elimination_array = elimination_array.clone();

            handlers.push(thread::spawn(move || {
                for _ in 0..item_count {
                    let mut strategy = DefaultStrategy::new();
                    while elimination_array.exchange_pop(&mut strategy).is_err() {}
                }
            }))
        }

        for handler in handlers {
            handler.join().unwrap();
        }
    }
}
