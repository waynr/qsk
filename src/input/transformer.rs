use evdev_rs::enums;
use evdev_rs::InputEvent;
use log::debug;

pub enum ControlCode {
    InputEvent(InputEvent),
    Exit,
}

pub struct Passthrough {}

impl InputTransformer for Passthrough {
    fn transform(&mut self, ie: &InputEvent) -> Option<Vec<ControlCode>> {
        match &ie.event_code {
            enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PAUSE) => Some(vec![ControlCode::Exit]),
            enums::EventCode::EV_KEY(_) => {
                debug!("{:?} {:?}", ie.event_code, ie.value);
                Some(vec![ControlCode::InputEvent(InputEvent {
                    time: ie.time.clone(),
                    event_code: ie.event_code.clone(),
                    event_type: ie.event_type.clone(),
                    value: ie.value.clone(),
                })])
            }
            _ => None,
        }
    }
}

pub trait InputTransformer {
    fn transform(&mut self, ie: &InputEvent) -> Option<Vec<ControlCode>>;
}
