use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use crate::control_code::{ControlCode, LayerRef};
use crate::errors::Result;
use crate::events::{InputEvent, EventCode, KeyCode, KeyCode::*, KeyState::*};
use crate::layers::{Layer, Layers};

/// An `InputTransformer` that passes through all input events it receives save for `KC_PAUSE`,
/// which it translates to `ControlCode::Exit`.
pub struct Passthrough {}

impl InputTransformer for Passthrough {
    fn transform(&mut self, e: InputEvent) -> Option<Vec<ControlCode>> {
        match e.code {
            EventCode::KeyCode(KC_PAUSE) => Some(vec![ControlCode::Exit]),
            _ => Some(vec![ControlCode::InputEvent(e)]),
        }
    }
}

pub trait InputTransformer {
    fn transform(&mut self, e: InputEvent) -> Option<Vec<ControlCode>>;
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

/// LayerComposer is the "top-level" type involved in `qsk`'s layered approach to keyboard
/// remapping. It works by iterating over the `Layer`s that it composes and applying the
/// transformation from the first active layer it finds to the given `InputEvent`.
pub struct LayerComposer {
    base: Box<dyn InputTransformer + Send>,
    layers: Layers,
    timers: HashMap<KeyCode, SystemTime>,

    nower: Box<dyn Nower + Send>,
}

impl LayerComposer {
    pub fn from_layers(layers: Vec<Layer>) -> Result<LayerComposer> {
        let composer = LayerComposer {
            base: Box::new(Passthrough {}),
            layers: layers.into(),
            timers: HashMap::new(),
            nower: Box::new(RealNower {}),
        };

        Ok(composer)
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

    fn key_up_and_down(&self, k: KeyCode) -> Vec<ControlCode> {
        let now = self.now();
        let now_plus = now + Duration::from_micros(1);
        vec![
            ControlCode::InputEvent(InputEvent {
                time: now,
                code: EventCode::KeyCode(k),
                state: Down,
            }),
            ControlCode::InputEvent(InputEvent {
                time: now_plus,
                code: EventCode::KeyCode(k),
                state: Up,
            }),
        ]
    }

    fn handle_control_codes(
        &mut self,
        e: &InputEvent,
        ccs: Vec<ControlCode>,
    ) -> Option<Vec<ControlCode>> {
        let mut output: Vec<ControlCode> = Vec::new();
        for cc in ccs {
            match cc {
                ControlCode::TapToggle(ref layer_ref, key) => match (e.state, self.timers.get(&key)) {
                    (Down, None) => {
                        self.timers.insert(key, self.now());
                    }
                    (Held, Some(t)) => {
                        if self.duration_since(*t) > Duration::from_millis(180) {
                            self.activate_layer(layer_ref);
                            self.timers.remove(&key);
                        }
                    }
                    (Up, None) => {
                        if self.is_layer_active(layer_ref) {
                            self.deactivate_layer(layer_ref);
                            self.timers.remove(&key);
                        } else {
                            self.key_up_and_down(key)
                                .iter()
                                .for_each(|cc| output.push(cc.clone()));
                        }
                    }
                    (Up, Some(t)) => {
                        if self.duration_since(*t) < Duration::from_millis(180) {
                            self.key_up_and_down(key)
                                .iter()
                                .for_each(|cc| output.push(cc.clone()));
                        }
                        self.deactivate_layer(layer_ref);
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

    fn is_layer_active(&mut self, lr: &LayerRef) -> bool {
        match lr {
            LayerRef::ByIndex(index) => {
                self.layers[*index].active
            },
            LayerRef::ByName(name) => {
                self.layers.get_mut(name).unwrap().active
            },
        }
    }

    fn activate_layer(&mut self, lr: &LayerRef) {
        self.set_layer_active(lr, true)
    }

    fn deactivate_layer(&mut self, lr: &LayerRef) {
        self.set_layer_active(lr, false)
    }

    fn set_layer_active(&mut self, lr: &LayerRef, to: bool) {
        match lr {
            LayerRef::ByIndex(index) => {
                self.layers[*index].active = to
            },
            LayerRef::ByName(name) => {
                self.layers.get_mut(name).unwrap().active = to
            },
        };
    }

    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        self.layers.iter()
    }
}

impl InputTransformer for LayerComposer {
    fn transform(&mut self, e: InputEvent) -> Option<Vec<ControlCode>> {
        for l in &mut self.layers.iter_mut().rev() {
            match l.transform(e) {
                Some(ccs) => return self.handle_control_codes(&e, ccs),
                None => continue,
            }
        }
        self.base.transform(e)
    }
}

#[cfg(test)]
mod layer_composer {
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    use galvanic_assert::matchers::collection::*;
    use galvanic_assert::matchers::*;
    use galvanic_assert::*;
    use maplit::hashmap;

    use super::*;
    use crate::KeyState;

    impl LayerComposer {
        fn key(&self, kc: KeyCode, ks: KeyState) -> InputEvent {
            InputEvent {
                time: self.nower.now(),
                code: EventCode::KeyCode(kc),
                state: ks,
            }
        }

        fn validate_single(&mut self, input: InputEvent, output: Option<InputEvent>) {
            let result = self.transform(input);
            match output {
                None => assert_that!(&result, eq(None)),
                Some(e) => {
                    let expect = vec![ControlCode::InputEvent(e)];
                    assert_that!(&result.unwrap(), contains_in_order(expect));
                }
            };
        }

        fn validate_multiple(&mut self, input: InputEvent, output: Vec<ControlCode>) {
            assert_that!(&self.transform(input).unwrap(), contains_in_order(output));
        }
    }

    pub fn key(k: KeyCode) -> Vec<ControlCode> { vec![ControlCode::KeyMap(k)] }

    pub fn tap_toggle(layer: usize, kc: KeyCode) -> Vec<ControlCode> {
        vec![ControlCode::TapToggle(LayerRef::ByIndex(layer), kc)]
    }

    pub fn tap_toggle_by_name(name: String, kc: KeyCode) -> Vec<ControlCode> {
        vec![ControlCode::TapToggle(LayerRef::ByName(name), kc)]
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

    fn test_layer_composer() -> (LayerComposer, FakeNow) {
        let mut layers = Vec::with_capacity(8);

        layers.insert(
            LAYERS::HomerowCodeRight.into(),
            Layer::from_hashmap(
                "control".to_string(),
                hashmap!(
                    KC_F => tap_toggle(LAYERS::Navigation.into(), KC_F),
                    KC_D => tap_toggle_by_name("navigation".to_string(), KC_D),
                ),
                true,
            ),
        );

        layers.insert(
            LAYERS::Navigation.into(),
            Layer::from_hashmap(
                "navigation".to_string(),
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

        let fake_now = FakeNow::new();
        (
            LayerComposer {
                base: Box::new(Passthrough {}),
                layers: layers.into(),
                timers: HashMap::new(),
                nower: Box::new(fake_now.clone()),
            },
            fake_now,
        )
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
        let (mut th, _) = test_layer_composer();
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));

        th.validate_single(th.key(KC_E, Down), Some(th.key(KC_E, Down)));
        th.validate_single(th.key(KC_E, Up), Some(th.key(KC_E, Up)));

        th.validate_single(th.key(KC_K, Down), Some(th.key(KC_K, Down)));
        th.validate_single(th.key(KC_K, Up), Some(th.key(KC_K, Up)));

        th.validate_single(th.key(KC_J, Down), Some(th.key(KC_J, Down)));
        th.validate_single(th.key(KC_J, Up), Some(th.key(KC_J, Up)));
    }

    #[test]
    fn tap_toggle_toggle_by_layer_name() {
        let (mut th, fake_now) = test_layer_composer();
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));

        // initial button down of a tap toggle key should not produce any characters and should not
        // set the toggle layer to active
        th.validate_single(th.key(KC_D, Down), None);
        assert_that!(&th.layers[1].active, eq(false));

        // layer doesn't get set to active until both after the next Held key fter the tap
        // toggle timeout
        fake_now.adjust_now(Duration::from_millis(1000));
        assert_that!(&th.layers[1].active, eq(false));
        th.validate_single(th.key(KC_D, Held), None);
        assert_that!(&th.layers[1].active, eq(true));

        // once layer is active, key transformation should take place based on definitions in the
        // activated layer
        th.validate_single(th.key(KC_J, Down), Some(th.key(KC_DOWN, Down)));
        th.validate_single(th.key(KC_J, Up), Some(th.key(KC_DOWN, Up)));

        // if layer is toggled, releasing tap toggle key after tap toggle timeout should result in
        // no keyboard events and should result in the layer being disabled once again
        th.validate_single(th.key(KC_D, Up), None);
        assert_that!(&th.layers[1].active, eq(false));
        th.validate_single(th.key(KC_J, Down), Some(th.key(KC_J, Down)));
        th.validate_single(th.key(KC_J, Up), Some(th.key(KC_J, Up)));
    }

    #[test]
    fn tap_toggle_toggle() {
        let (mut th, fake_now) = test_layer_composer();
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));

        // initial button down of a tap toggle key should not produce any characters and should not
        // set the toggle layer to active
        th.validate_single(th.key(KC_F, Down), None);
        assert_that!(&th.layers[1].active, eq(false));

        // layer doesn't get set to active until both after the next Held key fter the tap
        // toggle timeout
        fake_now.adjust_now(Duration::from_millis(1000));
        assert_that!(&th.layers[1].active, eq(false));
        th.validate_single(th.key(KC_F, Held), None);
        assert_that!(&th.layers[1].active, eq(true));

        // once layer is active, key transformation should take place based on definitions in the
        // activated layer
        th.validate_single(th.key(KC_J, Down), Some(th.key(KC_DOWN, Down)));
        th.validate_single(th.key(KC_J, Up), Some(th.key(KC_DOWN, Up)));

        // if layer is toggled, releasing tap toggle key after tap toggle timeout should result in
        // no keyboard events and should result in the layer being disabled once again
        th.validate_single(th.key(KC_F, Up), None);
        assert_that!(&th.layers[1].active, eq(false));
        th.validate_single(th.key(KC_J, Down), Some(th.key(KC_J, Down)));
        th.validate_single(th.key(KC_J, Up), Some(th.key(KC_J, Up)));
    }

    #[test]
    #[ignore]
    // TODO: try to remember what i was going to test here over a year ago...
    fn tap_toggle_regression_() {
        assert!(false);
        let (mut th, _) = test_layer_composer();
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));

        // initial button down of a tap toggle key should not produce any characters and should not
        // set the toggle layer to active
        th.validate_single(th.key(KC_F, Down), None);
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));
    }

    #[test]
    #[ignore]
    // TODO: try to remember what i was going to test here over a year ago...
    fn tap_toggle_tap_short_circuits_timeout() {
        assert!(false);
        let (mut th, fake_now) = test_layer_composer();
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));

        // if we type from the layer in question within the timeout the layer is activated
        th.validate_single(th.key(KC_F, Down), None);
        fake_now.adjust_now(Duration::from_millis(10));
        th.validate_multiple(
            th.key(KC_F, Up),
            vec![
                ControlCode::InputEvent(th.key(KC_F, Down)),
                ControlCode::InputEvent(th.key(KC_F, Up)),
            ],
        );
    }

    #[test]
    fn tap_toggle_tap() {
        let (mut th, _) = test_layer_composer();
        let mut expected: Vec<ControlCode> = Vec::new();
        assert_that!(&th.layers[0].active, eq(true));
        assert_that!(&th.layers[1].active, eq(false));

        // if we release the key within the tap toggle timeout, then we should get the tapped key's
        // usual output in sequence
        th.validate_single(th.key(KC_F, Down), None);

        let down = th.key(KC_F, Down);
        let mut up = th.key(KC_F, Up);
        up.time = down.time + Duration::from_micros(1);
        expected.push(ControlCode::InputEvent(down));
        expected.push(ControlCode::InputEvent(up));
        th.validate_multiple(th.key(KC_F, Up), expected);
    }

    #[test]
    fn key_up_and_down() {
        let (th, _) = test_layer_composer();
        let mut expected: Vec<ControlCode> = Vec::new();

        let down = th.key(KC_F, Down);
        let mut up = th.key(KC_F, Up);
        up.time = down.time + Duration::from_micros(1);
        expected.push(ControlCode::InputEvent(down));
        expected.push(ControlCode::InputEvent(up));

        let actual = th.key_up_and_down(KC_F);
        assert_that!(&actual, contains_in_order(expected));
    }
}
