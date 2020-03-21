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
        return true;
    }
}

impl treiber_stack::PopStrategy for DefaultStrategy {
    fn try_pop(&mut self) -> bool {
        if self.treiber_stack_pop_cnt == 1 {
            self.treiber_stack_pop_cnt = 0;
            return false;
        }

        self.treiber_stack_pop_cnt += 1;
        return true;
    }
}

impl elimination_array::PushStrategy for DefaultStrategy {}

impl elimination_array::PopStrategy for DefaultStrategy {
    fn try_pop(&mut self) -> bool {
        if self.elimination_array_pop_cnt == 1 {
            self.elimination_array_pop_cnt = 0;
            return false;
        }

        self.elimination_array_pop_cnt += 1;
        return true;
    }
}

impl exchanger::PushStrategy for DefaultStrategy {
    fn try_start_exchange(&mut self) -> bool {
        if self.exchanger_start_push_cnt > 10 {
            self.exchanger_start_push_cnt = 0;
            return false;
        }

        self.exchanger_start_push_cnt += 1;
        return true;
    }

    fn retry_check_exchanged(&mut self) -> bool {
        if self.exchanger_retry_check_success_cnt > 10 {
            self.exchanger_retry_check_success_cnt = 0;
            return false;
        }

        self.exchanger_retry_check_success_cnt += 1;
        return true;
    }
}

impl exchanger::PopStrategy for DefaultStrategy {
    fn try_exchange(&mut self) -> bool {
        if self.exchanger_try_pop_cnt > 10 {
            self.exchanger_try_pop_cnt = 0;
            return false;
        }

        self.exchanger_try_pop_cnt += 1;
        return true;
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
        return true;
    }
}

impl treiber_stack::PopStrategy for NoEliminationStrategy {
    fn try_pop(&mut self) -> bool {
        if self.treiber_stack_pop_cnt == 1 {
            self.treiber_stack_pop_cnt = 0;
            return false;
        }

        self.treiber_stack_pop_cnt += 1;
        return true;
    }
}

impl elimination_array::PushStrategy for NoEliminationStrategy {}

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
