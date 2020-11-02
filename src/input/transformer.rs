use log::debug;
use maplit::hashmap;
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
}

impl Layer {
    fn new() -> Self {
        Layer {
            map: HashMap::new(),
        }
    }

    fn transform(&mut self, e: event::KeyboardEvent) -> Option<Vec<ControlCode>> {
        None
    }
}

pub struct LayerComposer {
    base: Box<dyn InputTransformer + Send>,
    layers: Vec<Layer>,
    top_layer: usize,
}

enum LAYERS {
    HomerowCodeRight = 0,
    Navigation = 1,
}

impl LAYERS {
    fn to_usize(self) -> usize {
        self as usize
    }
}

fn tap_toggle(key: event::KeyCode, layer: LAYERS) -> Vec<ControlCode> {
    Vec::new()
}

fn key(k: event::KeyboardEvent) -> Vec<ControlCode> {
    vec![ControlCode::KeyboardEvent(k)]
}

impl LayerComposer {
    pub fn new() -> Self {
        let mut layers = Vec::with_capacity(8);

        layers.insert(
            LAYERS::HomerowCodeRight.to_usize(),
            Layer {
                map: hashmap!(
                    KC_F => tap_toggle(KC_F, LAYERS::Navigation)
                ),
            },
        );

        layers.insert(
            LAYERS::Navigation.to_usize(),
            Layer {
                map: hashmap!(
                    KC_Y => key(KC_HOME),
                    KC_U => key(KC_PAGEDOWN),
                    KC_I => key(KC_PAGEUP),
                    KC_O => key(KC_END),
                    KC_H => key(KC_LEFT),
                    KC_J => key(KC_DOWN),
                    KC_K => key(KC_UP),
                    KC_SEMICOLON => key(KC_RIGHT),
                ),
            },
        );

        LayerComposer {
            base: Box::new(Passthrough {}),
            layers: layers,
            top_layer: 0,
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
