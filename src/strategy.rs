//! A Strategy is a way to configure the behavior of a [`super::Stack`] at
//! compile time.
//!
//! By default a [`super::Stack`] uses the [`DefaultStrategy`]. Instead one can
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
//
// TODO: Rename this to BackAndForthStrategy.
#[derive(Default)]
pub struct DefaultStrategy {
    // TODO: usize is a bit big on 64bit machines, no?
    treiber_stack_push_cnt: usize,
    treiber_stack_pop_cnt: usize,

    elimination_array_push_cnt: usize,
    elimination_array_pop_cnt: usize,

    exchanger_start_push_cnt: usize,
    exchanger_retry_check_success_cnt: usize,
    exchanger_try_pop_cnt: usize,
}

impl DefaultStrategy {
    pub fn new() -> Self {
        DefaultStrategy::default()
    }
}

impl StackPushStrategy for DefaultStrategy {
    fn new() -> Self {
        DefaultStrategy::new()
    }

    fn use_elimination_array(&mut self) -> bool {
        true
    }
}

impl StackPopStrategy for DefaultStrategy {
    fn new() -> Self {
        DefaultStrategy::new()
    }

    fn use_elimination_array(&mut self) -> bool {
        true
    }
}

impl treiber_stack::PushStrategy for DefaultStrategy {
    fn try_push(&mut self) -> bool {
        if self.treiber_stack_push_cnt == 1 {
            self.treiber_stack_push_cnt = 0;
            return false;
        }

        self.treiber_stack_push_cnt += 1;
        true
    }
}

impl treiber_stack::PopStrategy for DefaultStrategy {
    fn try_pop(&mut self) -> bool {
        if self.treiber_stack_pop_cnt == 1 {
            self.treiber_stack_pop_cnt = 0;
            return false;
        }

        self.treiber_stack_pop_cnt += 1;
        true
    }
}

impl elimination_array::PushStrategy for DefaultStrategy {
    fn try_push(&mut self) -> bool {
        if self.elimination_array_push_cnt == 1 {
            self.elimination_array_push_cnt = 0;
            return false;
        }

        self.elimination_array_push_cnt += 1;
        true
    }
}

impl elimination_array::PopStrategy for DefaultStrategy {
    fn try_pop(&mut self) -> bool {
        if self.elimination_array_pop_cnt == 1 {
            self.elimination_array_pop_cnt = 0;
            return false;
        }

        self.elimination_array_pop_cnt += 1;
        true
    }
}

impl exchanger::PushStrategy for DefaultStrategy {
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

impl exchanger::PopStrategy for DefaultStrategy {
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

// TODO: Hopefully eventually an adaptive retry strategy.
#[derive(Default)]
pub struct RetryStrategy {
    // TODO: usize is a bit big on 64bit machines, no?
    treiber_stack_push_cnt: usize,
    treiber_stack_pop_cnt: usize,

    elimination_array_push_cnt: usize,
    elimination_array_pop_cnt: usize,

    exchanger_try_start_exchange_cnt: usize,
    exchanger_retry_check_exchanged_cnt: usize,
    exchanger_try_pop_exchange_cnt: usize,
}

impl RetryStrategy {
    fn new() -> Self {
        RetryStrategy::default()
    }
}

impl StackPushStrategy for RetryStrategy {
    fn new() -> Self {
        RetryStrategy::new()
    }

    fn use_elimination_array(&mut self) -> bool {
        true
    }
}

impl StackPopStrategy for RetryStrategy {
    fn new() -> Self {
        RetryStrategy::new()
    }

    fn use_elimination_array(&mut self) -> bool {
        true
    }
}

impl treiber_stack::PushStrategy for RetryStrategy {
    // Try push to Treiber stack at most once. Failing on Treiber stack implies
    // congestion which is best resolved via elimination array.
    fn try_push(&mut self) -> bool {
        if self.treiber_stack_push_cnt == 1 {
            self.treiber_stack_push_cnt = 0;
            return false;
        }

        self.treiber_stack_push_cnt += 1;
        true
    }
}

impl treiber_stack::PopStrategy for RetryStrategy {
    // Try pop from Treiber stack at most once. Failing on Treiber stack implies
    // congestion which is best resolved via elimination array.
    fn try_pop(&mut self) -> bool {
        if self.treiber_stack_pop_cnt == 1 {
            self.treiber_stack_pop_cnt = 0;
            return false;
        }

        self.treiber_stack_pop_cnt += 1;
        true
    }
}

impl elimination_array::PushStrategy for RetryStrategy {
    fn try_push(&mut self) -> bool {
        if self.elimination_array_push_cnt == 1 {
            self.elimination_array_push_cnt = 0;
            return false;
        }

        self.elimination_array_push_cnt += 1;
        true
    }
}

impl elimination_array::PopStrategy for RetryStrategy {
    // Try out 3 different exchangers before going back to the Treiber stack.
    //
    // Taken from page 260: Moir, Mark, et al. "Using elimination to implement
    // scalable and lock-free fifo queues." Proceedings of the seventeenth
    // annual ACM symposium on Parallelism in algorithms and architectures.
    // 2005.
    fn try_pop(&mut self) -> bool {
        if self.elimination_array_pop_cnt == 3 {
            self.elimination_array_pop_cnt = 0;
            return false;
        }

        self.elimination_array_pop_cnt += 1;
        true
    }
}

impl exchanger::PushStrategy for RetryStrategy {
    // Try to exchange a put on an exchanger at most once. Failure implies usage
    // by a different push operation.
    fn try_start_exchange(&mut self) -> bool {
        if self.exchanger_try_start_exchange_cnt == 1 {
            self.exchanger_try_start_exchange_cnt = 0;
            return false;
        }

        self.exchanger_try_start_exchange_cnt += 1;
        true
    }

    // Wait for a pop operation for up to 50 atomic loads.
    fn retry_check_exchanged(&mut self) -> bool {
        if self.exchanger_retry_check_exchanged_cnt == 50 {
            self.exchanger_retry_check_exchanged_cnt = 0;
            return false;
        }

        self.exchanger_retry_check_exchanged_cnt += 1;
        true
    }
}

impl exchanger::PopStrategy for RetryStrategy {
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
}
