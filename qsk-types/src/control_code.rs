use serde::{Deserialize, Serialize};
use crate::events::{InputEvent, KeyCode};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ControlCode {
    InputEvent(InputEvent),
    KeyMap(KeyCode),
    TapToggle(usize, KeyCode),
    TapToggleByName(String, KeyCode),
    Exit,
}

