use crate::{PopStrategy as StackPopStrategy, PushStrategy as StackPushStrategy};

/// Represents the default strategy aiming for good average performance.
pub struct DefaultStrategy {}

impl DefaultStrategy {
    fn new() -> Self {
        DefaultStrategy {}
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

// Strategy to have Stack use the Treiber Stack only and not elude to the
// elimination array on contention.
pub struct NoEliminationStrategy {}

impl NoEliminationStrategy {
    fn new() -> Self {
        NoEliminationStrategy {}
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
