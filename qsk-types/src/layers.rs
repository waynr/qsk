use std::collections::HashMap;

use crate::control_code::ControlCode;
use crate::events::{InputEvent, EventCode, KeyCode};

pub struct Layer {
    name: String,
    map: HashMap<EventCode, Vec<ControlCode>>,
    pub(crate) active: bool,
}

impl Layer {
    pub fn from_hashmap(name: String, map: HashMap<KeyCode, Vec<ControlCode>>, active: bool) -> Layer {
        let mut new_map = HashMap::with_capacity(map.len());
        map.iter().for_each(|(k, v)| {
            new_map.insert(EventCode::KeyCode(*k), v.clone());
        });
        Layer {
            name,
            map: new_map,
            active,
        }
    }

    pub(crate) fn transform(&mut self, e: InputEvent) -> Option<Vec<ControlCode>> {
        match (self.map.get(&e.code), self.active) {
            (Some(ccs), true) => {
                let mut output: Vec<ControlCode> = Vec::new();
                for cc in ccs {
                    match cc {
                        ControlCode::KeyMap(kc) => {
                            let mut cloned = e.clone();
                            cloned.code = EventCode::KeyCode(*kc);
                            output.push(ControlCode::InputEvent(cloned));
                        }
                        _ => output.push(cc.clone()),
                    }
                }
                Some(output)
            }
            (Some(_), false) => None,
            (None, _) => None,
        }
    }
}
