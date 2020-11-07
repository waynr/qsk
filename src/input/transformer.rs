use log::debug;
use maplit::hashmap;
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

use super::super::input::event;
use super::super::input::event::KeyCode::*;
use super::super::input::event::KeyState::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlCode {
    KeyboardEvent(event::KeyboardEvent),
    KeyMap(event::KeyCode),
    TapToggle(usize, event::KeyCode),
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
    fn transform(&mut self, e: event::KeyboardEvent) -> Option<Vec<ControlCode>> {
        match (self.map.get(&e.code), self.active) {
            (Some(ccs), true) => {
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
            (Some(_), false) => None,
            (None, _) => None,
        }
    }
}

pub struct LayerComposer<T>
where
    T: Fn() -> SystemTime,
{
    base: Box<dyn InputTransformer + Send>,
    layers: Vec<Layer>,
    timers: HashMap<event::KeyCode, Instant>,

    now_fn: T,
}

#[derive(Clone, Debug, PartialEq, Copy)]
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
    vec![ControlCode::TapToggle(layer.to_usize(), kc)]
}

impl<T> LayerComposer<T>
where
    T: Fn() -> SystemTime,
{
    pub fn new(now_fn: T) -> LayerComposer<T> {
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
            timers: HashMap::new(),
            now_fn: now_fn,
        }
    }

    fn now(&self) -> SystemTime {
        (self.now_fn)()
    }

    fn key_up_and_down(&self, k: event::KeyCode) -> Vec<ControlCode> {
        vec![
            ControlCode::KeyboardEvent(event::KeyboardEvent {
                time: self.now(),
                code: k,
                state: Down,
            }),
            ControlCode::KeyboardEvent(event::KeyboardEvent {
                time: self.now(),
                code: k,
                state: Up,
            }),
        ]
    }

    fn handle_control_codes(
        &mut self,
        e: &event::KeyboardEvent,
        ccs: Vec<ControlCode>,
    ) -> Option<Vec<ControlCode>> {
        let mut output: Vec<ControlCode> = Vec::new();
        for cc in ccs {
            match cc {
                // TODO: implement tap toggle timing calculation to differentiate between a tap and
                // a toggle.
                ControlCode::TapToggle(layer, key) => match (e.state, self.timers.get(&key)) {
                    (Down, None) => {
                        self.timers.insert(key, Instant::now());
                    }
                    (Held, Some(t)) => {
                        if Instant::now().duration_since(*t) > Duration::from_millis(180) {
                            self.layers[layer].active = true;
                            self.timers.remove(&key);
                        }
                    }
                    (Up, None) => {
                        if self.layers[layer].active {
                            self.layers[layer].active = false;
                            self.timers.remove(&key);
                        } else {
                            self.key_up_and_down(key)
                                .iter()
                                .for_each(|cc| output.push(*cc));
                        }
                    }
                    (Up, Some(t)) => {
                        if Instant::now().duration_since(*t) < Duration::from_millis(180) {
                            self.key_up_and_down(key)
                                .iter()
                                .for_each(|cc| output.push(*cc));
                        }
                        self.layers[layer].active = false;
                        self.timers.remove(&key);
                    }
                    (_, _) => output.push(cc),
                },
                _ => output.push(cc),
            }
        }
        match output[..] {
            [] => None,
            _ => Some(output),
        }
    }
}

impl<T> InputTransformer for LayerComposer<T>
where
    T: Fn() -> SystemTime,
{
    fn transform(&mut self, e: event::KeyboardEvent) -> Option<Vec<ControlCode>> {
        for l in &mut self.layers.iter_mut().rev() {
            match l.transform(e) {
                Some(ccs) => return self.handle_control_codes(&e, ccs),
                None => continue,
            }
        }
        self.base.transform(e)
    }
}

// test outline
//
// test simultaneous layer activation
// test tap toggle timing
//
#[cfg(test)]
mod layer_composer {
    use super::*;
    use galvanic_assert::matchers::collection::*;
    use galvanic_assert::matchers::*;
    use galvanic_assert::*;
    use std::time::SystemTime;

    impl<T> LayerComposer<T>
    where
        T: Fn() -> SystemTime,
    {
        fn ke(&self, kc: event::KeyCode, ks: event::KeyState) -> event::KeyboardEvent {
            event::KeyboardEvent {
                time: (self.now_fn)(),
                code: kc,
                state: ks,
            }
        }

        fn validate_single(
            &mut self,
            input: event::KeyboardEvent,
            output: Option<event::KeyboardEvent>,
        ) {
            let result = self.transform(input);
            match output {
                None => assert_that!(&result, eq(None)),
                Some(e) => {
                    let expect = vec![ControlCode::KeyboardEvent(e)];
                    assert_that!(&result.unwrap(), contains_in_order(expect));
                }
            };
        }

        fn validate_multiple(&mut self, input: event::KeyboardEvent, output: Vec<ControlCode>) {
            let result = self.transform(input);
            assert_that!(&self.transform(input).unwrap(), contains_in_order(output));
        }
    }

    #[test]
    fn passthrough_no_active_layers() {
        let now = SystemTime::now();
        let mut th = LayerComposer::new(|| now);

        th.validate_single(th.ke(KC_E, Down), Some(th.ke(KC_E, Down)));
        th.validate_single(th.ke(KC_E, Up), Some(th.ke(KC_E, Up)));

        th.validate_single(th.ke(KC_K, Down), Some(th.ke(KC_K, Down)));
        th.validate_single(th.ke(KC_K, Up), Some(th.ke(KC_K, Up)));

        th.validate_single(th.ke(KC_J, Down), Some(th.ke(KC_J, Down)));
        th.validate_single(th.ke(KC_J, Up), Some(th.ke(KC_J, Up)));

        th.validate_single(th.ke(KC_F, Down), None);
        th.validate_multiple(
            th.ke(KC_F, Up),
            vec![
                ControlCode::KeyboardEvent(th.ke(KC_F, Down)),
                ControlCode::KeyboardEvent(th.ke(KC_F, Up)),
            ],
        );

        //th.validate_single(th.ke(KC_F, Down), None);
    }

    #[test]
    fn toggle() {}
}
