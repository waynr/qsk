use log::debug;

use super::super::input::event;

pub enum ControlCode {
    KeyboardEvent(event::KeyboardEvent),
    DeactivateLayer(event::KeyCode),
    Exit,
}

pub struct Passthrough {}

impl InputTransformer for Passthrough {
    fn transform(&mut self, e: event::KeyboardEvent) -> Option<Vec<ControlCode>> {
        match e.code {
            event::KeyCode::KC_PAUSE => Some(vec![ControlCode::Exit]),
            _ => {
                debug!("{:?} {:?}", e.code, e.state);
                Some(vec![ControlCode::KeyboardEvent(e)])
            }
        }
    }
}

pub trait InputTransformer {
    fn transform(&mut self, e: event::KeyboardEvent) -> Option<Vec<ControlCode>>;
}

pub struct Composer {
    base: Box<dyn InputTransformer + Send>,
    active: Vec<Box<dyn InputTransformer + Send>>,
    deferred_actions: Vec<ControlCode>,
}

impl Composer {
    pub fn new() -> Self {
        Composer {
            base: Box::new(Passthrough {}),
            active: Vec::new(),
            deferred_actions: Vec::new(),
        }
    }
}

impl InputTransformer for Composer {
    fn transform(&mut self, e: event::KeyboardEvent) -> Option<Vec<ControlCode>> {
        for t in &mut self.active {
            match t.transform(e) {
                Some(v) => return Some(v),
                None => continue,
            }
        }
        self.base.transform(e)
    }
}
