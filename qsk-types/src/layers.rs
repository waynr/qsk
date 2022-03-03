use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use std::slice::IterMut;

use crate::control_code::ControlCode;
use crate::events::{InputEvent, EventCode, KeyCode};

#[derive(Clone)]
pub struct KeyMap(HashMap<EventCode, Vec<ControlCode>>);

impl Index<EventCode> for KeyMap {
    type Output = Vec<ControlCode>;

    fn index(&self, index: EventCode) -> &Self::Output {
        &self.0[&index]
    }
}

#[derive(Clone)]
pub struct Layer {
    pub name: String,
    map: KeyMap,
    pub active: bool,
}

impl Layer {
    pub fn from_hashmap(name: String, map: HashMap<KeyCode, Vec<ControlCode>>, active: bool) -> Layer {
        Layer {
            name,
            map: KeyMap(map
                .iter()
                .map(|(k, v)| (EventCode::KeyCode(*k), v.clone()) )
                .collect()),
            active,
        }
    }

    pub(crate) fn transform(&mut self, e: InputEvent) -> Option<Vec<ControlCode>> {
        match (self.map.0.get(&e.code), self.active) {
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

    pub fn activate(&mut self) {
        self.active = true
    }

    pub fn iter(&self) -> impl Iterator<Item = (&EventCode, &Vec<ControlCode>)> {
        self.map.0.iter()
    }
}

pub struct Layers {
    vec: Vec<Layer>,
    map: HashMap<String, usize>,
}

impl Index<usize> for Layers {
    type Output = Layer;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}

impl IndexMut<usize> for Layers {
    fn index_mut(& mut self, index: usize) -> &mut Self::Output {
        &mut self.vec[index]
    }
}

impl From<Vec<Layer>> for Layers {
    fn from(vec: Vec<Layer>) -> Self {
        let map: HashMap<String, usize> = vec.iter()
            .enumerate()
            .map(|(i, layer)| (layer.name.clone(), i))
            .collect();
        Self {
            vec,
            map,
        }
    }
}

impl Layers {
    pub(crate) fn iter_mut(&mut self) -> IterMut<Layer> {
        self.vec.iter_mut()
    }

    pub(crate) fn get_mut(&mut self, key: &str) -> Option<&mut Layer> {
        if let Some(idx) = self.map.get_mut(key) {
            return Some(&mut self.vec[*idx])
        }
        None
    }

    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        self.vec.iter()
    }
}
