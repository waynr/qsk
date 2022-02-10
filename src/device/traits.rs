use crate::errors::Result;
use crate::events::InputEvent;

pub trait InputEventSource: Send {
    fn recv(&mut self) -> Result<InputEvent>;
}

pub trait InputEventSink: Send {
    fn send(&mut self, e: InputEvent) -> Result<()>;
}
