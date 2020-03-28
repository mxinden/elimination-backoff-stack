//! A Strategy is a way to configure the behavior of a [`super::Stack`] at
//! compile time.
//!
//! By default a [`super::Stack`] uses the [`ExpRetryStrategy`]. Instead one can
//! initiate a [`super::Stack`] with e.g. the [`NoEliminationStrategy`] to get
//! a classic Treiber stack only.
//!
//! ```rust
//! # use elimination_backoff_stack::Stack;
//! # use elimination_backoff_stack::strategy::NoEliminationStrategy;
//! Stack::<
//!   String,
//!   NoEliminationStrategy,
//!   NoEliminationStrategy,
//! >::new();
//! ```
//!
//! Why at compile time?
//!
//! To reduce the overhead introduced through isolated behavior management by
//! enabling the compiler to do all kinds of things, e.g. constant folding.

use crate::{
    elimination_array, exchanger, treiber_stack, PopStrategy as StackPopStrategy,
    PushStrategy as StackPushStrategy,
};

/// Represents the default strategy aiming for good average performance.
#[derive(Default)]
pub struct BackAndForthStrategy {
    // TODO: usize is a bit big on 64bit machines, no?
    treiber_stack_push_cnt: usize,
    treiber_stack_pop_cnt: usize,

    elimination_array_push_cnt: usize,
    elimination_array_pop_cnt: usize,

    exchanger_start_push_cnt: usize,
    exchanger_retry_check_success_cnt: usize,
    exchanger_try_pop_cnt: usize,
}

impl BackAndForthStrategy {
    pub fn new() -> Self {
        BackAndForthStrategy::default()
    }
}

impl StackPushStrategy for BackAndForthStrategy {
    fn new() -> Self {
        BackAndForthStrategy::new()
    }

    fn use_elimination_array(&mut self) -> bool {
        true
    }
}

impl StackPopStrategy for BackAndForthStrategy {
    fn new() -> Self {
        BackAndForthStrategy::new()
    }

    fn use_elimination_array(&mut self) -> bool {
        true
    }
}

impl treiber_stack::PushStrategy for BackAndForthStrategy {
    fn try_push(&mut self) -> bool {
        if self.treiber_stack_push_cnt == 1 {
            self.treiber_stack_push_cnt = 0;
            return false;
        }

        self.treiber_stack_push_cnt += 1;
        true
    }
}

impl treiber_stack::PopStrategy for BackAndForthStrategy {
    fn try_pop(&mut self) -> bool {
        if self.treiber_stack_pop_cnt == 1 {
            self.treiber_stack_pop_cnt = 0;
            return false;
        }

        self.treiber_stack_pop_cnt += 1;
        true
    }
}

impl elimination_array::PushStrategy for BackAndForthStrategy {
    fn try_push(&mut self) -> bool {
        if self.elimination_array_push_cnt == 1 {
            self.elimination_array_push_cnt = 0;
            return false;
        }

        self.elimination_array_push_cnt += 1;
        true
    }
}

impl elimination_array::PopStrategy for BackAndForthStrategy {
    fn try_pop(&mut self) -> bool {
        if self.elimination_array_pop_cnt == 1 {
            self.elimination_array_pop_cnt = 0;
            return false;
        }

        self.elimination_array_pop_cnt += 1;
        true
    }
}

impl exchanger::PushStrategy for BackAndForthStrategy {
    fn try_start_exchange(&mut self) -> bool {
        if self.exchanger_start_push_cnt > 10 {
            self.exchanger_start_push_cnt = 0;
            return false;
        }

        self.exchanger_start_push_cnt += 1;
        true
    }

    fn retry_check_exchanged(&mut self) -> bool {
        if self.exchanger_retry_check_success_cnt > 10 {
            self.exchanger_retry_check_success_cnt = 0;
            return false;
        }

        self.exchanger_retry_check_success_cnt += 1;
        true
    }
}

impl exchanger::PopStrategy for BackAndForthStrategy {
    fn try_exchange(&mut self) -> bool {
        if self.exchanger_try_pop_cnt > 10 {
            self.exchanger_try_pop_cnt = 0;
            return false;
        }

        self.exchanger_try_pop_cnt += 1;
        true
    }
}

/// Strategy to have Stack use the Treiber stack only and not elude to the
/// elimination array on contention.
#[derive(Default)]
pub struct NoEliminationStrategy {
    treiber_stack_push_cnt: usize,
    treiber_stack_pop_cnt: usize,
}

impl NoEliminationStrategy {
    fn new() -> Self {
        NoEliminationStrategy::default()
    }
}

impl StackPushStrategy for NoEliminationStrategy {
    fn new() -> Self {
        NoEliminationStrategy::new()
    }

    fn use_elimination_array(&mut self) -> bool {
        false
    }
}

impl StackPopStrategy for NoEliminationStrategy {
    fn new() -> Self {
        NoEliminationStrategy::new()
    }

    fn use_elimination_array(&mut self) -> bool {
        false
    }
}

impl treiber_stack::PushStrategy for NoEliminationStrategy {
    fn try_push(&mut self) -> bool {
        if self.treiber_stack_push_cnt == 1 {
            self.treiber_stack_push_cnt = 0;
            return false;
        }

        self.treiber_stack_push_cnt += 1;
        true
    }
}

impl treiber_stack::PopStrategy for NoEliminationStrategy {
    fn try_pop(&mut self) -> bool {
        if self.treiber_stack_pop_cnt == 1 {
            self.treiber_stack_pop_cnt = 0;
            return false;
        }

        self.treiber_stack_pop_cnt += 1;
        true
    }
}

impl elimination_array::PushStrategy for NoEliminationStrategy {
    fn try_push(&mut self) -> bool {
        false
    }
}

impl elimination_array::PopStrategy for NoEliminationStrategy {
    fn try_pop(&mut self) -> bool {
        false
    }
}

impl exchanger::PushStrategy for NoEliminationStrategy {
    fn try_start_exchange(&mut self) -> bool {
        false
    }

    fn retry_check_exchanged(&mut self) -> bool {
        false
    }
}

impl exchanger::PopStrategy for NoEliminationStrategy {
    fn try_exchange(&mut self) -> bool {
        false
    }
}

/// Strategy retrying failed operations with exponential back-off in both space
/// and time.
///
/// Back-off in space: At first try a subset of all elimination array
/// exchangers, exponentially increasing on congestion.
///
/// Back-off in time: Retry elimination array on congestion and Treiber stack on
/// disappearing of congestion.
#[derive(Default)]
pub struct ExpRetryStrategy {
    retry_exponent: u8,

    // TODO: usize is a bit big on 64bit machines, no?
    treiber_stack_push_cnt: usize,
    treiber_stack_pop_cnt: usize,

    elimination_array_push_cnt: usize,
    elimination_array_pop_cnt: usize,

    exchanger_try_start_exchange_cnt: usize,
    exchanger_retry_check_exchanged_cnt: usize,
    exchanger_try_pop_exchange_cnt: usize,
}

const MAX_RETRY_EXPONENT: u8 = 5;

impl ExpRetryStrategy {
    pub fn new() -> Self {
        ExpRetryStrategy::default()
    }
}

impl StackPushStrategy for ExpRetryStrategy {
    fn new() -> Self {
        ExpRetryStrategy::new()
    }

    fn use_elimination_array(&mut self) -> bool {
        true
    }
}

impl StackPopStrategy for ExpRetryStrategy {
    fn new() -> Self {
        ExpRetryStrategy::new()
    }

    fn use_elimination_array(&mut self) -> bool {
        true
    }
}

impl treiber_stack::PushStrategy for ExpRetryStrategy {
    // Try push to Treiber stack at most once. Failing on Treiber stack implies
    // congestion which is best resolved via elimination array.
    //
    // TODO: Maybe retry once. Should improve the case of light congestion.
    fn try_push(&mut self) -> bool {
        if self.treiber_stack_push_cnt == 1 {
            // Increase retry exponent due to congestion.
            self.retry_exponent = (self.retry_exponent + 1).min(MAX_RETRY_EXPONENT);

            self.treiber_stack_push_cnt = 0;

            return false;
        }

        self.treiber_stack_push_cnt += 1;
        true
    }
}

impl treiber_stack::PopStrategy for ExpRetryStrategy {
    // Try pop from Treiber stack at most once. Failing on Treiber stack implies
    // congestion which is best resolved via elimination array.
    //
    // TODO: Maybe retry once. Should improve the case of light congestion.
    fn try_pop(&mut self) -> bool {
        if self.treiber_stack_pop_cnt == 1 {
            // Increase retry exponent due to congestion.
            self.retry_exponent = (self.retry_exponent + 1).min(MAX_RETRY_EXPONENT);

            self.treiber_stack_pop_cnt = 0;

            return false;
        }

        self.treiber_stack_pop_cnt += 1;
        true
    }
}

impl elimination_array::PushStrategy for ExpRetryStrategy {
    // Try at least 2 times multiplied by 2 each time congestion occurs.
    fn try_push(&mut self) -> bool {
        if self.elimination_array_push_cnt >= (2 << self.retry_exponent) {
            self.elimination_array_push_cnt = 0;
            return false;
        }

        self.elimination_array_push_cnt += 1;
        true
    }

    fn num_exchangers(&mut self, total: usize) -> usize {
        (1 << self.retry_exponent).min(total)
    }
}

impl elimination_array::PopStrategy for ExpRetryStrategy {
    // Try at least 2 times multiplied by 2 each time congestion occurs.
    //
    // See page 260 for more research: Moir, Mark, et al. "Using elimination to
    // implement scalable and lock-free fifo queues." Proceedings of the
    // seventeenth annual ACM symposium on Parallelism in algorithms and
    // architectures. 2005.
    fn try_pop(&mut self) -> bool {
        if self.elimination_array_pop_cnt >= (2 << self.retry_exponent) {
            self.elimination_array_pop_cnt = 0;
            return false;
        }

        self.elimination_array_pop_cnt += 1;
        true
    }

    fn num_exchangers(&mut self, total: usize) -> usize {
        elimination_array::PushStrategy::num_exchangers(self, total)
    }
}

impl exchanger::PushStrategy for ExpRetryStrategy {
    // Try to exchange a put on an exchanger at most once. Failure implies usage
    // by a different push operation. Thus never retry the same exchanger but
    // try a different one.
    fn try_start_exchange(&mut self) -> bool {
        if self.exchanger_try_start_exchange_cnt == 1 {
            // Given that there was congestion, increase the retry exponent.
            self.retry_exponent = (self.retry_exponent + 1).min(MAX_RETRY_EXPONENT);

            self.exchanger_try_start_exchange_cnt = 0;

            return false;
        }

        self.exchanger_try_start_exchange_cnt += 1;
        true
    }

    // Wait for a pop operation for up to 50 atomic loads.
    fn retry_check_exchanged(&mut self) -> bool {
        // TODO: Should this grow exponentially with contention? 1 on 8 threads
        // and 100 for 128 threads worked well in the past.
        for _ in 0..(self.retry_exponent) {
            std::sync::atomic::spin_loop_hint();
        }

        // TODO: Should this grow exponentially with contention? 10 on 8 threads
        // and 50 on 128 threads worked well in the past.
        if self.exchanger_retry_check_exchanged_cnt == (10 * self.retry_exponent) as usize {
            // No pop operation exchanging with this push operation signals less
            // congestion. Thus decreasing the retry exponent.
            self.retry_exponent = self.retry_exponent.saturating_sub(2);

            self.exchanger_retry_check_exchanged_cnt = 0;

            return false;
        }

        self.exchanger_retry_check_exchanged_cnt += 1;
        true
    }
}

impl exchanger::PopStrategy for ExpRetryStrategy {
    // Failure on pop implies that either (a) there is no concurrent push
    // operation in progress on the exchanger (b) the concurrent push operation
    // was already matched with a pop operation. Thus best to try a different
    // exchanger.
    fn try_exchange(&mut self) -> bool {
        if self.exchanger_try_pop_exchange_cnt == 1 {
            self.exchanger_try_pop_exchange_cnt = 0;
            return false;
        }

        self.exchanger_try_pop_exchange_cnt += 1;
        true
    }

    fn on_contention(&mut self) {
        self.retry_exponent = (self.retry_exponent + 1).max(MAX_RETRY_EXPONENT);
    }

    fn on_no_contention(&mut self) {
        self.retry_exponent = self.retry_exponent.saturating_sub(2);
    }
}
