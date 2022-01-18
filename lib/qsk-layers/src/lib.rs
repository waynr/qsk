use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use qsk_events as event;
use qsk_events::{
    EventCode, KeyCode::*, KeyState::*,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlCode {
    InputEvent(event::InputEvent),
    KeyMap(event::KeyCode),
    TapToggle(usize, event::KeyCode),
    Exit,
}

pub struct Passthrough {}

impl InputTransformer for Passthrough {
    fn transform(&mut self, e: event::InputEvent) -> Option<Vec<ControlCode>> {
        match e.code {
           EventCode::KeyCode(KC_PAUSE) => Some(vec![ControlCode::Exit]),
            _ => {
                Some(vec![ControlCode::InputEvent(e)])
            }
        }
    }
}

pub trait InputTransformer {
    fn transform(&mut self, e: event::InputEvent) -> Option<Vec<ControlCode>>;
}

pub struct Layer {
    map: HashMap<EventCode, Vec<ControlCode>>,
    active: bool,
}

impl Layer {
    pub fn from_hashmap(map: HashMap<event::KeyCode, Vec<ControlCode>>, active: bool) -> Layer {
        let mut new_map = HashMap::with_capacity(map.len());
        map.iter().for_each(|(k, v)| {
                new_map.insert(EventCode::KeyCode(*k), v.clone());
            });
        Layer { map: new_map, active }
    }

    fn transform(&mut self, e: event::InputEvent) -> Option<Vec<ControlCode>> {
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

trait Nower {
    fn now(&self) -> SystemTime;
}

struct RealNower {}

impl Nower for RealNower {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

pub struct LayerComposer {
    base: Box<dyn InputTransformer + Send>,
    layers: Vec<Layer>,
    timers: HashMap<event::KeyCode, SystemTime>,

    nower: Box<dyn Nower + Send>,
}

pub fn key(k: event::KeyCode) -> Vec<ControlCode> {
    vec![ControlCode::KeyMap(k)]
}

pub fn tap_toggle(layer: usize, kc: event::KeyCode) -> Vec<ControlCode> {
    vec![ControlCode::TapToggle(layer, kc)]
}

impl LayerComposer {
    pub fn from_layers(layers: Vec<Layer>) -> LayerComposer {
        LayerComposer {
            base: Box::new(Passthrough {}),
            layers,
            timers: HashMap::new(),
            nower: Box::new(RealNower {}),
        }
    }

    fn now(&self) -> SystemTime {
        self.nower.now()
    }

    fn duration_since(&self, t: SystemTime) -> Duration {
        match self.now().duration_since(t) {
            Ok(d) => d,
            Err(_) => Duration::new(0, 0),
        }
    }

    fn key_up_and_down(&self, k: event::KeyCode) -> Vec<ControlCode> {
        vec![
            ControlCode::InputEvent(event::InputEvent {
                time: self.now(),
                code: EventCode::KeyCode(k),
                state: Down,
            }),
            ControlCode::InputEvent(event::InputEvent {
                time: self.now(),
                code: EventCode::KeyCode(k),
                state: Up,
            }),
        ]
    }

    fn handle_control_codes(
        &mut self,
        e: &event::InputEvent,
        ccs: Vec<ControlCode>,
    ) -> Option<Vec<ControlCode>> {
        let mut output: Vec<ControlCode> = Vec::new();
        for cc in ccs {
            match cc {
                ControlCode::TapToggle(layer, key) => match (e.state, self.timers.get(&key)) {
                    (Down, None) => {
                        self.timers.insert(key, self.now());
                    }
                    (Held, Some(t)) => {
                        if self.duration_since(*t) > Duration::from_millis(180) {
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
                        if self.duration_since(*t) < Duration::from_millis(180) {
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

impl InputTransformer for LayerComposer {
    fn transform(&mut self, e: event::InputEvent) -> Option<Vec<ControlCode>> {
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
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    use maplit::hashmap;
    use galvanic_assert::matchers::collection::*;
    use galvanic_assert::matchers::*;
    use galvanic_assert::*;

    use super::*;

    impl LayerComposer {
        fn ke(&self, kc: event::KeyCode, ks: event::KeyState) -> event::InputEvent {
            event::InputEvent {
                time: self.nower.now(),
                code: EventCode::KeyCode(kc),
                state: ks,
            }
        }

        fn validate_single(
            &mut self,
            input: event::InputEvent,
            output: Option<event::InputEvent>,
        ) {
            let result = self.transform(input);
            match output {
                None => assert_that!(&result, eq(None)),
                Some(e) => {
                    let expect = vec![ControlCode::InputEvent(e)];
                    assert_that!(&result.unwrap(), contains_in_order(expect));
                }
            };
        }

        fn validate_multiple(&mut self, input: event::InputEvent, output: Vec<ControlCode>) {
            assert_that!(&self.transform(input).unwrap(), contains_in_order(output));
        }
    }

    #[derive(Clone)]
    struct FakeNow {
        t: Arc<Mutex<SystemTime>>,
    }

    impl FakeNow {
        fn new() -> Self {
            FakeNow {
                t: Arc::new(Mutex::new(SystemTime::now())),
            }
        }
        fn adjust_now(&self, by: Duration) {
            let mut mut_ref = self.t.lock().unwrap();
            *mut_ref += by;
        }
    }

    impl Nower for FakeNow {
        fn now(&self) -> SystemTime {
            self.t.lock().unwrap().clone()
        }
    }

    #[derive(Clone, Debug, PartialEq, Copy)]
    enum LAYERS {
        HomerowCodeRight = 0,
        Navigation = 1,
    }

    impl From<LAYERS> for usize {
        fn from(layer: LAYERS) -> usize {
            layer as usize
        }
    }

    fn test_layer_composer() -> LayerComposer {
        let mut layers = Vec::with_capacity(8);

        layers.insert(
            LAYERS::HomerowCodeRight.into(),
            Layer::from_hashmap(
                hashmap!(
                    KC_F => tap_toggle(LAYERS::Navigation.into(), KC_F)
                ),
                true,
            ),
        );

        layers.insert(
            LAYERS::Navigation.into(),
            Layer::from_hashmap(
                hashmap!(
                    KC_Y => key(KC_HOME),
                    KC_U => key(KC_PAGEDOWN),
                    KC_I => key(KC_PAGEUP),
                    KC_O => key(KC_END),
                    KC_H => key(KC_LEFT),
                    KC_J => key(KC_DOWN),
                    KC_K => key(KC_UP),
                    KC_SEMICOLON => key(KC_RIGHT),
                ),
                false,
            ),
        );

        LayerComposer {
            base: Box::new(Passthrough {}),
            layers,
            timers: HashMap::new(),
            nower: Box::new(RealNower {}),
        }
    }

    #[test]
    fn fake_now() {
        let fake_now = Box::new(FakeNow::new());
        let c_fake_now = fake_now.clone();

        let t1 = fake_now.now();
        fake_now.adjust_now(Duration::from_millis(1000));
        let mut t2 = fake_now.now();
        assert_eq!(t2, t1 + Duration::from_millis(1000));

        t2 = c_fake_now.now();
        assert_eq!(t2, t1 + Duration::from_millis(1000));
    }

    #[test]
    fn passthrough_no_active_layers() {
        let fake_now = Box::new(FakeNow::new());
        let mut th = test_layer_composer();
        th.nower = fake_now.clone();
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));

        th.validate_single(th.ke(KC_E, Down), Some(th.ke(KC_E, Down)));
        th.validate_single(th.ke(KC_E, Up), Some(th.ke(KC_E, Up)));

        th.validate_single(th.ke(KC_K, Down), Some(th.ke(KC_K, Down)));
        th.validate_single(th.ke(KC_K, Up), Some(th.ke(KC_K, Up)));

        th.validate_single(th.ke(KC_J, Down), Some(th.ke(KC_J, Down)));
        th.validate_single(th.ke(KC_J, Up), Some(th.ke(KC_J, Up)));
    }

    #[test]
    fn tap_toggle_toggle() {
        let fake_now = Box::new(FakeNow::new());
        let mut th = test_layer_composer();
        th.nower = fake_now.clone();
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));

        // initial button down of a tap toggle key should not produce any characters and should not
        // set the toggle layer to active
        th.validate_single(th.ke(KC_F, Down), None);
        assert_that!(&th.layers[1].active, eq(false));

        // layer doesn't get set to active until both after the next Held key event after the tap
        // toggle timeout
        fake_now.adjust_now(Duration::from_millis(1000));
        assert_that!(&th.layers[1].active, eq(false));
        th.validate_single(th.ke(KC_F, Held), None);
        assert_that!(&th.layers[1].active, eq(true));

        // once layer is active, key transformation should take place based on definitions in the
        // activated layer
        th.validate_single(th.ke(KC_J, Down), Some(th.ke(KC_DOWN, Down)));
        th.validate_single(th.ke(KC_J, Up), Some(th.ke(KC_DOWN, Up)));

        // if layer is toggled, releasing tap toggle key after tap toggle timeout should result in
        // no keyboard events and should result in the layer being disabled once again
        th.validate_single(th.ke(KC_F, Up), None);
        assert_that!(&th.layers[1].active, eq(false));
        th.validate_single(th.ke(KC_J, Down), Some(th.ke(KC_J, Down)));
        th.validate_single(th.ke(KC_J, Up), Some(th.ke(KC_J, Up)));
    }

    #[test]
    #[ignore]
    // TODO: try to remember what i was going to test here over a year ago...
    fn tap_toggle_regression_() {
        assert!(false);
        let fake_now = Box::new(FakeNow::new());
        let mut th = test_layer_composer();
        th.nower = fake_now.clone();
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));

        // initial button down of a tap toggle key should not produce any characters and should not
        // set the toggle layer to active
        th.validate_single(th.ke(KC_F, Down), None);
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));
    }

    #[test]
    #[ignore]
    // TODO: try to remember what i was going to test here over a year ago...
    fn tap_toggle_tap_short_circuits_timeout() {
        assert!(false);
        let fake_now = Box::new(FakeNow::new());
        let mut th = test_layer_composer();
        th.nower = fake_now.clone();
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));

        // if we type from the layer in question within the timeout the layer is activated
        th.validate_single(th.ke(KC_F, Down), None);
        fake_now.adjust_now(Duration::from_millis(10));
        th.validate_multiple(
            th.ke(KC_F, Up),
            vec![
                ControlCode::InputEvent(th.ke(KC_F, Down)),
                ControlCode::InputEvent(th.ke(KC_F, Up)),
            ],
        );
    }

    #[test]
    fn tap_toggle_tap() {
        let fake_now = Box::new(FakeNow::new());
        let mut th = test_layer_composer();
        th.nower = fake_now.clone();
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));

        // if we release the key within the tap toggle timeout, then we should get the tapped key's
        // usual output in sequence
        th.validate_single(th.ke(KC_F, Down), None);
        fake_now.adjust_now(Duration::from_millis(10));
        th.validate_multiple(
            th.ke(KC_F, Up),
            vec![
                ControlCode::InputEvent(th.ke(KC_F, Down)),
                ControlCode::InputEvent(th.ke(KC_F, Up)),
            ],
        );
    }
}
