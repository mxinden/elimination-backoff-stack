#[derive(Clone, Debug)]
pub(crate) enum Event {
    StartPush,
    StartEliminationArrayPush,
    StartExchangerPush,
    StartPop,
    StartEliminationArrayPop,
    StartExchangerPop,
    TryStack,
    TryEliminationArray,
    FinishPush,
    FinishPop,
}

pub(crate) fn print_padded(e: &Event) {
    let padding = match e {
        Event::StartPush => 0,
        Event::StartEliminationArrayPush => 2,
        Event::StartExchangerPush => 3,
        Event::StartPop => 0,
        Event::StartEliminationArrayPop => 2,
        Event::StartExchangerPop => 3,
        Event::TryStack => 1,
        Event::TryEliminationArray => 1,
        Event::FinishPush => 0,
        Event::FinishPop => 0,
    };

    for padding in 0..padding {
        print!("\t");
    }

    print!("{:?}\n", e);
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
