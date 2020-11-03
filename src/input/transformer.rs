use log::debug;
use maplit::hashmap;
use std::collections::HashMap;

use super::super::input::event;
use super::super::input::event::KeyCode::*;

#[derive(Clone)]
pub enum ControlCode {
    KeyboardEvent(event::KeyboardEvent),
    KeyMap(event::KeyCode),
    DeactivateLayer(event::KeyCode),
    TapToggle(LAYERS, event::KeyCode),
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
        match self.map.get(&e.code) {
            Some(ccs) => {
                let mut output: Vec<ControlCode> = Vec::new();
                for cc in ccs {
                    match cc {
                        ControlCode::KeyMap(kc) => {
                            let mut cloned = e.clone();
                            cloned.code = *kc;
                            output.push(ControlCode::KeyboardEvent(cloned));
                        }
                        _ => output.push(cc.clone()),
                    }
                }
                Some(output)
            }
            None => None,
        }
    }
}

pub struct LayerComposer {
    base: Box<dyn InputTransformer + Send>,
    layers: Vec<Layer>,
}

#[derive(Clone)]
enum LAYERS {
    HomerowCodeRight = 0,
    Navigation = 1,
}

impl LAYERS {
    fn to_usize(self) -> usize {
        self as usize
    }
}

fn key(k: event::KeyCode) -> Vec<ControlCode> {
    vec![ControlCode::KeyMap(k)]
}

fn tap_toggle(layer: LAYERS, kc: event::KeyCode) -> Vec<ControlCode> {
    vec![ControlCode::TapToggle(layer, kc)]
}

impl LayerComposer {
    pub fn new() -> Self {
        let mut layers = Vec::with_capacity(8);

        layers.insert(
            LAYERS::HomerowCodeRight.to_usize(),
            Layer {
                active: true,
                map: hashmap!(
                    KC_F => tap_toggle(LAYERS::Navigation, KC_F)
                ),
            },
        );

        layers.insert(
            LAYERS::Navigation.to_usize(),
            Layer {
                active: false,
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
        }
    }

    fn handle_control_codes(
        &mut self,
        e: &event::KeyboardEvent,
        ccs: Vec<ControlCode>,
    ) -> Vec<ControlCode> {
        let mut output: Vec<ControlCode> = Vec::new();
        for cc in ccs {
            match cc {
                // TODO: implement tap toggle timing calculation to differentiate between a tap and
                // a toggle.
                ControlCode::TapToggle(layer, key) => {
                    self.layers[layer.to_usize()].active = true;
                }
                _ => output.push(cc),
            }
        }
        output
    }
}

impl InputTransformer for LayerComposer {
    fn transform(&mut self, e: event::KeyboardEvent) -> Option<Vec<ControlCode>> {
        for l in &mut self.layers.iter_mut().rev() {
            match l.transform(e) {
                Some(ccs) => return Some(self.handle_control_codes(&e, ccs)),
                None => continue,
            }
        }
        self.base.transform(e)
    }
}
