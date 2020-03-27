#[derive(Clone, Debug)]
pub(crate) enum Event {
    StartPush,
    StartEliminationArrayPush,
    StartPop,
    StartEliminationArrayPop,
    TryStack,
    TryEliminationArray,
    FinishPush,
    FinishPop,
}

pub(crate) trait EventRecorder {
    fn record(&mut self, e: Event);
}

pub(crate) struct NoOpRecorder {}

impl EventRecorder for NoOpRecorder {
    fn record(&mut self, _event: Event) {}
}

impl EventRecorder for Vec<Event> {
    fn record(&mut self, event: Event) {
        self.push(event);
    }
}
