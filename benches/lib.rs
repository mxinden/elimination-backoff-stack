use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use elimination_backoff_stack::{
    strategy::{NoEliminationStrategy, RetryStrategy},
    PopStrategy, PushStrategy, Stack as EliminationBackoffStack,
};
use std::sync::{Arc, Mutex};
use std::thread;

trait Stack<T: Send>: Send + Sync + Clone {
    fn push(&self, item: T);
    fn pop(&self) -> Option<T>;
}

impl<T, PushS, PopS> Stack<T> for Arc<EliminationBackoffStack<T, PushS, PopS>>
where
    T: Send + Sync,
    PushS: PushStrategy + Send + Sync,
    PopS: PopStrategy + Send + Sync,
{
    fn push(&self, item: T) {
        EliminationBackoffStack::push(self, item);
    }

    fn pop(&self) -> Option<T> {
        EliminationBackoffStack::pop(self)
    }
}

impl<T: Send> Stack<T> for Arc<Mutex<Vec<T>>> {
    fn push(&self, item: T) {
        self.lock().unwrap().push(item);
    }

    fn pop(&self) -> Option<T> {
        self.lock().unwrap().pop()
    }
}

fn bench_stacks(c: &mut Criterion) {
    fn benchmark(stack: impl Stack<Vec<u8>> + 'static, threads: usize, item_count: u64) {
        let item = "my_test_item".as_bytes().to_vec();

        let mut handlers = vec![];

        for _ in 0..(threads / 2) {
            let push_stack = stack.clone();
            let item = item.clone();
            handlers.push(thread::spawn(move || {
                for _ in 0..item_count {
                    push_stack.push(item.clone());
                }
            }));

            let pop_stack = stack.clone();
            handlers.push(thread::spawn(move || {
                for _ in 0..item_count {
                    while pop_stack.pop().is_some() {}
                }
            }))
        }

        for handler in handlers {
            handler.join().unwrap();
        }
    }

    let mut group = c.benchmark_group("stacks");

    let item_count = 1_000;
    // group.throughput(Throughput::Elements(item_count));

    for i in [1, 2, 4, 8].iter() {
        group.bench_with_input(BenchmarkId::new("Arc<Mutex<Vec<_>>", i), i, |b, i| {
            b.iter(|| {
                let stack = Arc::new(Mutex::new(vec![]));
                benchmark(stack, *i, item_count);
            })
        });
        group.bench_with_input(
            BenchmarkId::new("Arc<EliminationBackoffStack<_>>", i),
            i,
            |b, i| {
                b.iter(|| {
                    let stack = Arc::new(EliminationBackoffStack::<_>::new());
                    benchmark(stack, *i, item_count);
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Arc<EliminationBackoffStack<_, NoEliminationStrategy>>", i),
            i,
            |b, i| {
                b.iter(|| {
                    let stack = Arc::new(EliminationBackoffStack::<
                        _,
                        NoEliminationStrategy,
                        NoEliminationStrategy,
                    >::new());
                    benchmark(stack, *i, item_count);
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Arc<EliminationBackoffStack<_, RetryStrategy>>", i),
            i,
            |b, i| {
                b.iter(|| {
                    let stack = Arc::new(EliminationBackoffStack::<
                        _,
                        RetryStrategy,
                        RetryStrategy,
                    >::new());
                    benchmark(stack, *i, item_count);
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_stacks);
criterion_main!(benches);
