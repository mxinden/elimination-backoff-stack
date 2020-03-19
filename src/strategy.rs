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
