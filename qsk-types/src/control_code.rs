use serde::{Deserialize, Serialize};
use crate::events::{InputEvent, KeyCode};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LayerRef {
    ByIndex(usize),
    ByName(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ControlCode {
    InputEvent(InputEvent),
    KeyMap(KeyCode),
    TapToggle(LayerRef, KeyCode),
    Exit,
}

