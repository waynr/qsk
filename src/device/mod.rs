pub mod linux;

use super::input::event;

pub trait InputEventSource {
    fn next_event(&self) -> Result<event::KeyboardEvent, Box<dyn std::error::Error>>;
}

pub trait InputEventSink {
    fn send_event(&self, e: &event::KeyboardEvent) -> Result<(), Box<dyn std::error::Error>>;
}
