use serde::{Deserialize, Serialize};
use crate::events::{InputEvent, KeyCode};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ControlCode {
    InputEvent(InputEvent),
    KeyMap(KeyCode),
    TapToggle(usize, KeyCode),
    Exit,
}

