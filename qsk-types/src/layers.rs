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
        let mut new_map = HashMap::with_capacity(map.len());
        map.iter().for_each(|(k, v)| {
            new_map.insert(EventCode::KeyCode(*k), v.clone());
        });
        Layer {
            name,
            map: KeyMap(new_map),
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
    // TODO: learn how to implement a self-referential struct and make the key here a reference to
    // a layer's name and the Layer a reference to the named layer.
    // map: HashMap<String, Layer>,
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
        // let mut map: HashMap<String, Layer> = HashMap::new();
        // for layer in vec.clone().iter() {
        //     map.insert(layer.name.clone(), layer.clone());
        // }
        Self {
            vec,
            // map,
        }
    }
}

impl Layers {
    pub(crate) fn iter_mut(&mut self) -> IterMut<Layer> {
        self.vec.iter_mut()
    }

    pub(crate) fn get_mut(&mut self, key: &str) -> Option<&mut Layer> {
        // self.map.get_mut(key)
        for layer in self.vec.iter_mut() {
            if layer.name.as_str() == key {
                return Some(layer)
            }
        }
        None
    }

    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        self.vec.iter()
    }
}
