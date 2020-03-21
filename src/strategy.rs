use crate::{
    elimination_array, treiber_stack, PopStrategy as StackPopStrategy,
    PushStrategy as StackPushStrategy,
};

/// Represents the default strategy aiming for good average performance.
#[derive(Default)]
pub struct DefaultStrategy {
    // TODO: usize is a bit big on 64bit machines, no?
    treiber_stack_push_cnt: usize,
    treiber_stack_pop_cnt: usize,

    elimination_array_pop_cnt: usize,
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

impl elimination_array::PopStrategy for NoEliminationStrategy {
    fn try_pop(&mut self) -> bool {
        false
    }
}
