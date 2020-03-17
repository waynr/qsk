use log::debug;
use std::collections::HashMap;

use super::super::input::event;
use super::super::input::event::KeyCode::*;

#[derive(Clone)]
pub enum ControlCode {
    KeyboardEvent(event::KeyboardEvent),
    DeactivateLayer(event::KeyCode),
    Exit,
}

pub struct Passthrough {}

impl InputTransformer for Passthrough {
    fn transform(&mut self, e: event::KeyboardEvent) -> Option<Vec<ControlCode>> {
        match e.code {
            KC_PAUSE => Some(vec![ControlCode::Exit]),
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

struct Layer {
    map: HashMap<event::KeyCode, Vec<ControlCode>>,
    active: bool,
}

impl Layer {
    fn new() -> Self {
        Layer {
            map: HashMap::new(),
            active: false,
        }
    }

    fn transform(&mut self, e: event::KeyboardEvent) -> Option<Vec<ControlCode>> {
        if !self.active {
            return None;
        }
        None
    }
}

pub struct LayerComposer {
    base: Box<dyn InputTransformer + Send>,
    layers: Vec<Layer>,
}

enum LAYERS {
    BASE = 0,
    NAVIGATION = 1,
}

impl LAYERS {
    fn to_usize(self) -> usize {
        self as usize
    }
}

fn tap_toggle(key: event::KeyCode, layer: LAYERS) -> Vec<ControlCode> {
    Vec::new()
}

impl LayerComposer {
    pub fn new() -> Self {
        let mut layers = Vec::with_capacity(8);

        layers[LAYERS::BASE.to_usize()] = Layer {
            map: [(KC_F, tap_toggle(KC_F, LAYERS::NAVIGATION))]
                .iter()
                .cloned()
                .collect(),
            active: true,
        };

        layers[LAYERS::NAVIGATION.to_usize()] = Layer {
            map: [(KC_F, tap_toggle(KC_F, LAYERS::NAVIGATION))]
                .iter()
                .cloned()
                .collect(),
            active: false,
        };

        LayerComposer {
            base: Box::new(Passthrough {}),
            layers: layers,
        }
    }

    fn handle_control_codes(&mut self, ccs: Vec<ControlCode>) -> Vec<ControlCode> {
        ccs
    }
}

impl InputTransformer for LayerComposer {
    fn transform(&mut self, e: event::KeyboardEvent) -> Option<Vec<ControlCode>> {
        for l in &mut self.layers {
            match l.transform(e) {
                Some(ccs) => return Some(self.handle_control_codes(ccs)),
                None => continue,
            }
        }
        self.base.transform(e)
    }
}
