use crate::event::{Event, EventRecorder, NoOpRecorder};
use crate::exchanger::{self, Exchanger};
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

    pub(crate) fn exchange_push<S: PushStrategy, R: EventRecorder>(
        &self,
        item: T,
        strategy: &mut S,
        recorder: &mut R,
    ) -> Result<(), T> {
        recorder.record(Event::StartEliminationArrayPush);

        let mut item = item;

        while strategy.try_push() {
            let num_exchangers = strategy.num_exchangers(self.exchangers.len());
            match self
                .rnd_exchanger(num_exchangers)
                .exchange_push(item, strategy, recorder)
            {
                Ok(()) => return Ok(()),
                Err(i) => item = i,
            }
        }

        Err(item)
    }

    pub(crate) fn exchange_pop<S: PopStrategy, R: EventRecorder>(
        &self,
        strategy: &mut S,
        recorder: &mut R,
    ) -> Result<T, ()> {
        recorder.record(Event::StartEliminationArrayPop);

        while strategy.try_pop() {
            let num_exchangers = strategy.num_exchangers(self.exchangers.len());
            if let Ok(item) = self
                .rnd_exchanger(num_exchangers)
                .exchange_pop(strategy, recorder)
            {
                return Ok(item);
            }
        }

        Err(())
    }

    fn rnd_exchanger(&self, range: usize) -> &Exchanger<T> {
        let i = thread_rng().gen_range(0, range);
        &self.exchangers[i]
    }
}

pub trait PushStrategy: exchanger::PushStrategy {
    fn try_push(&mut self) -> bool;

    /// Decide how many of the `total` exchangers should be considered. On low
    /// contention one could only use the first x exchangers to increase the
    /// exchange-success rate.
    fn num_exchangers(&mut self, total: usize) -> usize {
        total
    }
}

pub trait PopStrategy: exchanger::PopStrategy {
    fn try_pop(&mut self) -> bool;

    /// Decide how many of the `total` exchangers should be considered. On low
    /// contention one could only use the first x exchangers to increase the
    /// exchange-success rate.
    fn num_exchangers(&mut self, total: usize) -> usize {
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::DefaultStrategy;
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
                let mut recorder = NoOpRecorder {};
                for _ in 0..item_count {
                    let mut strategy = DefaultStrategy::new();
                    while elimination_array
                        .exchange_push((), &mut strategy, &mut recorder)
                        .is_err()
                    {}
                }
            }))
        }

        // Pop threads.
        for _ in 0..(num_cpus::get() / 2) {
            let elimination_array = elimination_array.clone();

            handlers.push(thread::spawn(move || {
                let mut recorder = NoOpRecorder {};
                for _ in 0..item_count {
                    let mut strategy = DefaultStrategy::new();
                    while elimination_array
                        .exchange_pop(&mut strategy, &mut recorder)
                        .is_err()
                    {}
                }
            }))
        }

        for handler in handlers {
            handler.join().unwrap();
        }
    }
}
