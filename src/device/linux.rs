use std::convert::TryFrom;
use std::sync::Arc;
use std::fs::File;
use std::sync::Mutex;
use std::path::PathBuf;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use evdev_rs;
use evdev_rs::enums;
use evdev_rs::GrabMode;
use evdev_rs::InputEvent;
use evdev_rs::TimeVal;
use log::error;

#[path = "../error.rs"]
mod error;
use error::Result;

use qsk_events as event;

pub struct Device {
    inner: Arc<Mutex<evdev_rs::Device>>,
}

unsafe impl Send for Device {}

impl Device {
    pub fn from_path(path: PathBuf) -> Result<Device> {
        let f = File::open(path)?;
        let mut d = evdev_rs::Device::new().unwrap();
        d.set_fd(f)?;
        d.grab(GrabMode::Grab)?;
        Ok(Device {
            inner: Arc::new(Mutex::new(d)),
        })
    }

    pub fn new_uinput_device(&self) -> Result<UInputDevice> {
        let guard = match self.inner.lock() {
            Ok(a) => a,
            Err(p_err) => {
                let g = p_err.into_inner();
                error!("recovered Device");
                g
            }
        };
        let d = evdev_rs::UInputDevice::create_from_device(&*guard)?;
        Ok(UInputDevice {
            inner: Arc::new(Mutex::new(d)),
        })
    }
}

impl event::InputEventSource for Device {
    fn recv(&self) -> std::result::Result<event::KeyboardEvent, Box<dyn std::error::Error + Send>> {
        let guard = match self.inner.lock() {
            Ok(a) => a,
            Err(p_err) => {
                let g = p_err.into_inner();
                error!("recovered Device");
                g
            }
        };
        match guard.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING) {
            Ok(ev) => Ok(ie_into_ke(ev.1)),
            Err(e) => Err(Box::new(e)),
        }
    }
}

pub struct UInputDevice {
    inner: Arc<Mutex<evdev_rs::UInputDevice>>,
}

unsafe impl Send for UInputDevice {}

impl event::InputEventSink for UInputDevice {
    fn send(&self, e: event::KeyboardEvent) -> std::result::Result<(), Box<dyn std::error::Error + Send>> {
        let guard = match self.inner.lock() {
            Ok(a) => a,
            Err(p_err) => {
                let g = p_err.into_inner();
                error!("recovered Device");
                g
            }
        };

        if let Some(ie) = ke_into_ie(e) {
            match guard.write_event(&ie) {
                Ok(_) => (),
                Err(e) => return Err(Box::new(e)),
            };
            let t: TimeVal;
            match TimeVal::try_from(e.time) {
                Ok(tv) => t = tv,
                Err(e) => return Err(Box::new(e)),
            }
            match guard.write_event(&InputEvent {
                time: t,
                event_type: enums::EventType::EV_SYN,
                event_code: enums::EventCode::EV_SYN(enums::EV_SYN::SYN_REPORT),
                value: 0,
            }) {
                Ok(_) => return Ok(()),
                Err(e) => return Err(Box::new(e)),
            }
        }
        Ok(())
    }
}

fn ke_into_ie(kv: event::KeyboardEvent) -> Option<evdev_rs::InputEvent> {
    match kc_into_ec(kv.code) {
        Some(ec) => {
            let d = match kv.time.duration_since(UNIX_EPOCH) {
                Ok(n) => n,
                Err(_) => Duration::new(0, 0),
            };
            Some(InputEvent {
                time: TimeVal {
                    tv_sec: d.as_secs() as i64,
                    tv_usec: d.subsec_micros() as i64,
                },
                event_type: enums::EventType::EV_KEY,
                event_code: ec,
                value: kv.state as i32,
            })
        }
        None => None,
    }
}

fn ie_into_ke(ev: evdev_rs::InputEvent) -> event::KeyboardEvent {
    event::KeyboardEvent {
        time: UNIX_EPOCH
            + Duration::new(ev.time.tv_sec as u64, ev.time.tv_usec as u32 * 1000 as u32),
        code: ec_into_kc(ev.event_code),
        state: i32_into_ks(ev.value),
    }
}

fn kc_into_ec(k: event::KeyCode) -> Option<enums::EventCode> {
    match enums::int_to_ev_key(k as u32) {
        Some(ev_key) => Some(enums::EventCode::EV_KEY(ev_key)),
        None => None,
    }
}

fn ec_into_kc(ec: enums::EventCode) -> event::KeyCode {
    match ec {
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RESERVED) => event::KeyCode::KC_RESERVED,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ESC) => event::KeyCode::KC_ESC,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_1) => event::KeyCode::KC_1,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_2) => event::KeyCode::KC_2,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_3) => event::KeyCode::KC_3,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_4) => event::KeyCode::KC_4,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_5) => event::KeyCode::KC_5,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_6) => event::KeyCode::KC_6,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_7) => event::KeyCode::KC_7,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_8) => event::KeyCode::KC_8,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_9) => event::KeyCode::KC_9,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_0) => event::KeyCode::KC_0,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MINUS) => event::KeyCode::KC_MINUS,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_EQUAL) => event::KeyCode::KC_EQUAL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BACKSPACE) => event::KeyCode::KC_BACKSPACE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TAB) => event::KeyCode::KC_TAB,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_Q) => event::KeyCode::KC_Q,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_W) => event::KeyCode::KC_W,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_E) => event::KeyCode::KC_E,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_R) => event::KeyCode::KC_R,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_T) => event::KeyCode::KC_T,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_Y) => event::KeyCode::KC_Y,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_U) => event::KeyCode::KC_U,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_I) => event::KeyCode::KC_I,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_O) => event::KeyCode::KC_O,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_P) => event::KeyCode::KC_P,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LEFTBRACE) => event::KeyCode::KC_LEFTBRACE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RIGHTBRACE) => event::KeyCode::KC_RIGHTBRACE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ENTER) => event::KeyCode::KC_ENTER,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LEFTCTRL) => event::KeyCode::KC_LEFTCTRL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_A) => event::KeyCode::KC_A,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_S) => event::KeyCode::KC_S,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_D) => event::KeyCode::KC_D,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F) => event::KeyCode::KC_F,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_G) => event::KeyCode::KC_G,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_H) => event::KeyCode::KC_H,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_J) => event::KeyCode::KC_J,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_K) => event::KeyCode::KC_K,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_L) => event::KeyCode::KC_L,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SEMICOLON) => event::KeyCode::KC_SEMICOLON,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_APOSTROPHE) => event::KeyCode::KC_APOSTROPHE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_GRAVE) => event::KeyCode::KC_GRAVE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LEFTSHIFT) => event::KeyCode::KC_LEFTSHIFT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BACKSLASH) => event::KeyCode::KC_BACKSLASH,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_Z) => event::KeyCode::KC_Z,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_X) => event::KeyCode::KC_X,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_C) => event::KeyCode::KC_C,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_V) => event::KeyCode::KC_V,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_B) => event::KeyCode::KC_B,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_N) => event::KeyCode::KC_N,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_M) => event::KeyCode::KC_M,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_COMMA) => event::KeyCode::KC_COMMA,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DOT) => event::KeyCode::KC_DOT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SLASH) => event::KeyCode::KC_SLASH,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RIGHTSHIFT) => event::KeyCode::KC_RIGHTSHIFT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KPASTERISK) => event::KeyCode::KC_KPASTERISK,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LEFTALT) => event::KeyCode::KC_LEFTALT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SPACE) => event::KeyCode::KC_SPACE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CAPSLOCK) => event::KeyCode::KC_CAPSLOCK,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F1) => event::KeyCode::KC_F1,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F2) => event::KeyCode::KC_F2,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F3) => event::KeyCode::KC_F3,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F4) => event::KeyCode::KC_F4,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F5) => event::KeyCode::KC_F5,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F6) => event::KeyCode::KC_F6,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F7) => event::KeyCode::KC_F7,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F8) => event::KeyCode::KC_F8,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F9) => event::KeyCode::KC_F9,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F10) => event::KeyCode::KC_F10,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMLOCK) => event::KeyCode::KC_NUMLOCK,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SCROLLLOCK) => event::KeyCode::KC_SCROLLLOCK,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KP7) => event::KeyCode::KC_KP7,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KP8) => event::KeyCode::KC_KP8,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KP9) => event::KeyCode::KC_KP9,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KPMINUS) => event::KeyCode::KC_KPMINUS,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KP4) => event::KeyCode::KC_KP4,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KP5) => event::KeyCode::KC_KP5,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KP6) => event::KeyCode::KC_KP6,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KPPLUS) => event::KeyCode::KC_KPPLUS,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KP1) => event::KeyCode::KC_KP1,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KP2) => event::KeyCode::KC_KP2,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KP3) => event::KeyCode::KC_KP3,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KP0) => event::KeyCode::KC_KP0,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KPDOT) => event::KeyCode::KC_KPDOT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ZENKAKUHANKAKU) => {
            event::KeyCode::KC_ZENKAKUHANKAKU
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_102ND) => event::KeyCode::KC_102ND,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F11) => event::KeyCode::KC_F11,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F12) => event::KeyCode::KC_F12,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RO) => event::KeyCode::KC_RO,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KATAKANA) => event::KeyCode::KC_KATAKANA,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_HIRAGANA) => event::KeyCode::KC_HIRAGANA,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_HENKAN) => event::KeyCode::KC_HENKAN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KATAKANAHIRAGANA) => {
            event::KeyCode::KC_KATAKANAHIRAGANA
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MUHENKAN) => event::KeyCode::KC_MUHENKAN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KPJPCOMMA) => event::KeyCode::KC_KPJPCOMMA,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KPENTER) => event::KeyCode::KC_KPENTER,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RIGHTCTRL) => event::KeyCode::KC_RIGHTCTRL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KPSLASH) => event::KeyCode::KC_KPSLASH,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SYSRQ) => event::KeyCode::KC_SYSRQ,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RIGHTALT) => event::KeyCode::KC_RIGHTALT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LINEFEED) => event::KeyCode::KC_LINEFEED,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_HOME) => event::KeyCode::KC_HOME,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_UP) => event::KeyCode::KC_UP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PAGEUP) => event::KeyCode::KC_PAGEUP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LEFT) => event::KeyCode::KC_LEFT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RIGHT) => event::KeyCode::KC_RIGHT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_END) => event::KeyCode::KC_END,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DOWN) => event::KeyCode::KC_DOWN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PAGEDOWN) => event::KeyCode::KC_PAGEDOWN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_INSERT) => event::KeyCode::KC_INSERT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DELETE) => event::KeyCode::KC_DELETE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MACRO) => event::KeyCode::KC_MACRO,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MUTE) => event::KeyCode::KC_MUTE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_VOLUMEDOWN) => event::KeyCode::KC_VOLUMEDOWN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_VOLUMEUP) => event::KeyCode::KC_VOLUMEUP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_POWER) => event::KeyCode::KC_POWER,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KPEQUAL) => event::KeyCode::KC_KPEQUAL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KPPLUSMINUS) => event::KeyCode::KC_KPPLUSMINUS,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PAUSE) => event::KeyCode::KC_PAUSE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SCALE) => event::KeyCode::KC_SCALE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KPCOMMA) => event::KeyCode::KC_KPCOMMA,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_HANGEUL) => event::KeyCode::KC_HANGEUL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_HANJA) => event::KeyCode::KC_HANJA,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_YEN) => event::KeyCode::KC_YEN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LEFTMETA) => event::KeyCode::KC_LEFTMETA,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RIGHTMETA) => event::KeyCode::KC_RIGHTMETA,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_COMPOSE) => event::KeyCode::KC_COMPOSE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_STOP) => event::KeyCode::KC_STOP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_AGAIN) => event::KeyCode::KC_AGAIN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PROPS) => event::KeyCode::KC_PROPS,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_UNDO) => event::KeyCode::KC_UNDO,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FRONT) => event::KeyCode::KC_FRONT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_COPY) => event::KeyCode::KC_COPY,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_OPEN) => event::KeyCode::KC_OPEN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PASTE) => event::KeyCode::KC_PASTE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FIND) => event::KeyCode::KC_FIND,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CUT) => event::KeyCode::KC_CUT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_HELP) => event::KeyCode::KC_HELP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MENU) => event::KeyCode::KC_MENU,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CALC) => event::KeyCode::KC_CALC,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SETUP) => event::KeyCode::KC_SETUP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SLEEP) => event::KeyCode::KC_SLEEP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_WAKEUP) => event::KeyCode::KC_WAKEUP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FILE) => event::KeyCode::KC_FILE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SENDFILE) => event::KeyCode::KC_SENDFILE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DELETEFILE) => event::KeyCode::KC_DELETEFILE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_XFER) => event::KeyCode::KC_XFER,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PROG1) => event::KeyCode::KC_PROG1,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PROG2) => event::KeyCode::KC_PROG2,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_WWW) => event::KeyCode::KC_WWW,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MSDOS) => event::KeyCode::KC_MSDOS,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_COFFEE) => event::KeyCode::KC_COFFEE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ROTATE_DISPLAY) => {
            event::KeyCode::KC_ROTATE_DISPLAY
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CYCLEWINDOWS) => {
            event::KeyCode::KC_CYCLEWINDOWS
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MAIL) => event::KeyCode::KC_MAIL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BOOKMARKS) => event::KeyCode::KC_BOOKMARKS,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_COMPUTER) => event::KeyCode::KC_COMPUTER,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BACK) => event::KeyCode::KC_BACK,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FORWARD) => event::KeyCode::KC_FORWARD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CLOSECD) => event::KeyCode::KC_CLOSECD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_EJECTCD) => event::KeyCode::KC_EJECTCD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_EJECTCLOSECD) => {
            event::KeyCode::KC_EJECTCLOSECD
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NEXTSONG) => event::KeyCode::KC_NEXTSONG,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PLAYPAUSE) => event::KeyCode::KC_PLAYPAUSE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PREVIOUSSONG) => {
            event::KeyCode::KC_PREVIOUSSONG
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_STOPCD) => event::KeyCode::KC_STOPCD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RECORD) => event::KeyCode::KC_RECORD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_REWIND) => event::KeyCode::KC_REWIND,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PHONE) => event::KeyCode::KC_PHONE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ISO) => event::KeyCode::KC_ISO,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CONFIG) => event::KeyCode::KC_CONFIG,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_HOMEPAGE) => event::KeyCode::KC_HOMEPAGE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_REFRESH) => event::KeyCode::KC_REFRESH,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_EXIT) => event::KeyCode::KC_EXIT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MOVE) => event::KeyCode::KC_MOVE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_EDIT) => event::KeyCode::KC_EDIT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SCROLLUP) => event::KeyCode::KC_SCROLLUP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SCROLLDOWN) => event::KeyCode::KC_SCROLLDOWN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KPLEFTPAREN) => event::KeyCode::KC_KPLEFTPAREN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KPRIGHTPAREN) => {
            event::KeyCode::KC_KPRIGHTPAREN
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NEW) => event::KeyCode::KC_NEW,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_REDO) => event::KeyCode::KC_REDO,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F13) => event::KeyCode::KC_F13,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F14) => event::KeyCode::KC_F14,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F15) => event::KeyCode::KC_F15,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F16) => event::KeyCode::KC_F16,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F17) => event::KeyCode::KC_F17,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F18) => event::KeyCode::KC_F18,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F19) => event::KeyCode::KC_F19,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F20) => event::KeyCode::KC_F20,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F21) => event::KeyCode::KC_F21,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F22) => event::KeyCode::KC_F22,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F23) => event::KeyCode::KC_F23,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_F24) => event::KeyCode::KC_F24,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PLAYCD) => event::KeyCode::KC_PLAYCD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PAUSECD) => event::KeyCode::KC_PAUSECD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PROG3) => event::KeyCode::KC_PROG3,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PROG4) => event::KeyCode::KC_PROG4,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DASHBOARD) => event::KeyCode::KC_DASHBOARD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SUSPEND) => event::KeyCode::KC_SUSPEND,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CLOSE) => event::KeyCode::KC_CLOSE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PLAY) => event::KeyCode::KC_PLAY,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FASTFORWARD) => event::KeyCode::KC_FASTFORWARD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BASSBOOST) => event::KeyCode::KC_BASSBOOST,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PRINT) => event::KeyCode::KC_PRINT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_HP) => event::KeyCode::KC_HP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CAMERA) => event::KeyCode::KC_CAMERA,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SOUND) => event::KeyCode::KC_SOUND,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_QUESTION) => event::KeyCode::KC_QUESTION,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_EMAIL) => event::KeyCode::KC_EMAIL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CHAT) => event::KeyCode::KC_CHAT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SEARCH) => event::KeyCode::KC_SEARCH,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CONNECT) => event::KeyCode::KC_CONNECT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FINANCE) => event::KeyCode::KC_FINANCE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SPORT) => event::KeyCode::KC_SPORT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SHOP) => event::KeyCode::KC_SHOP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ALTERASE) => event::KeyCode::KC_ALTERASE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CANCEL) => event::KeyCode::KC_CANCEL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRIGHTNESSDOWN) => {
            event::KeyCode::KC_BRIGHTNESSDOWN
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRIGHTNESSUP) => {
            event::KeyCode::KC_BRIGHTNESSUP
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MEDIA) => event::KeyCode::KC_MEDIA,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SWITCHVIDEOMODE) => {
            event::KeyCode::KC_SWITCHVIDEOMODE
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KBDILLUMTOGGLE) => {
            event::KeyCode::KC_KBDILLUMTOGGLE
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KBDILLUMDOWN) => {
            event::KeyCode::KC_KBDILLUMDOWN
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KBDILLUMUP) => event::KeyCode::KC_KBDILLUMUP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SEND) => event::KeyCode::KC_SEND,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_REPLY) => event::KeyCode::KC_REPLY,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FORWARDMAIL) => event::KeyCode::KC_FORWARDMAIL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SAVE) => event::KeyCode::KC_SAVE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DOCUMENTS) => event::KeyCode::KC_DOCUMENTS,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BATTERY) => event::KeyCode::KC_BATTERY,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BLUETOOTH) => event::KeyCode::KC_BLUETOOTH,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_WLAN) => event::KeyCode::KC_WLAN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_UWB) => event::KeyCode::KC_UWB,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_UNKNOWN) => event::KeyCode::KC_UNKNOWN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_VIDEO_NEXT) => event::KeyCode::KC_VIDEO_NEXT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_VIDEO_PREV) => event::KeyCode::KC_VIDEO_PREV,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRIGHTNESS_CYCLE) => {
            event::KeyCode::KC_BRIGHTNESS_CYCLE
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRIGHTNESS_AUTO) => {
            event::KeyCode::KC_BRIGHTNESS_AUTO
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DISPLAY_OFF) => event::KeyCode::KC_DISPLAY_OFF,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_WWAN) => event::KeyCode::KC_WWAN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RFKILL) => event::KeyCode::KC_RFKILL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MICMUTE) => event::KeyCode::KC_MICMUTE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_OK) => event::KeyCode::KC_OK,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SELECT) => event::KeyCode::KC_SELECT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_GOTO) => event::KeyCode::KC_GOTO,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CLEAR) => event::KeyCode::KC_CLEAR,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_POWER2) => event::KeyCode::KC_POWER2,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_OPTION) => event::KeyCode::KC_OPTION,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_INFO) => event::KeyCode::KC_INFO,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TIME) => event::KeyCode::KC_TIME,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_VENDOR) => event::KeyCode::KC_VENDOR,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ARCHIVE) => event::KeyCode::KC_ARCHIVE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PROGRAM) => event::KeyCode::KC_PROGRAM,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CHANNEL) => event::KeyCode::KC_CHANNEL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FAVORITES) => event::KeyCode::KC_FAVORITES,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_EPG) => event::KeyCode::KC_EPG,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PVR) => event::KeyCode::KC_PVR,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MHP) => event::KeyCode::KC_MHP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LANGUAGE) => event::KeyCode::KC_LANGUAGE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TITLE) => event::KeyCode::KC_TITLE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SUBTITLE) => event::KeyCode::KC_SUBTITLE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ANGLE) => event::KeyCode::KC_ANGLE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FULL_SCREEN) => event::KeyCode::KC_FULL_SCREEN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KEYBOARD) => event::KeyCode::KC_KEYBOARD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ASPECT_RATIO) => {
            event::KeyCode::KC_ASPECT_RATIO
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PC) => event::KeyCode::KC_PC,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TV) => event::KeyCode::KC_TV,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TV2) => event::KeyCode::KC_TV2,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_VCR) => event::KeyCode::KC_VCR,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_VCR2) => event::KeyCode::KC_VCR2,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SAT) => event::KeyCode::KC_SAT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SAT2) => event::KeyCode::KC_SAT2,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CD) => event::KeyCode::KC_CD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TAPE) => event::KeyCode::KC_TAPE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RADIO) => event::KeyCode::KC_RADIO,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TUNER) => event::KeyCode::KC_TUNER,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PLAYER) => event::KeyCode::KC_PLAYER,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TEXT) => event::KeyCode::KC_TEXT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DVD) => event::KeyCode::KC_DVD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_AUX) => event::KeyCode::KC_AUX,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MP3) => event::KeyCode::KC_MP3,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_AUDIO) => event::KeyCode::KC_AUDIO,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_VIDEO) => event::KeyCode::KC_VIDEO,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DIRECTORY) => event::KeyCode::KC_DIRECTORY,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LIST) => event::KeyCode::KC_LIST,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MEMO) => event::KeyCode::KC_MEMO,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CALENDAR) => event::KeyCode::KC_CALENDAR,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RED) => event::KeyCode::KC_RED,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_GREEN) => event::KeyCode::KC_GREEN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_YELLOW) => event::KeyCode::KC_YELLOW,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BLUE) => event::KeyCode::KC_BLUE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CHANNELUP) => event::KeyCode::KC_CHANNELUP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CHANNELDOWN) => event::KeyCode::KC_CHANNELDOWN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FIRST) => event::KeyCode::KC_FIRST,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LAST) => event::KeyCode::KC_LAST,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_AB) => event::KeyCode::KC_AB,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NEXT) => event::KeyCode::KC_NEXT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RESTART) => event::KeyCode::KC_RESTART,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SLOW) => event::KeyCode::KC_SLOW,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SHUFFLE) => event::KeyCode::KC_SHUFFLE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BREAK) => event::KeyCode::KC_BREAK,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PREVIOUS) => event::KeyCode::KC_PREVIOUS,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DIGITS) => event::KeyCode::KC_DIGITS,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TEEN) => event::KeyCode::KC_TEEN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TWEN) => event::KeyCode::KC_TWEN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_VIDEOPHONE) => event::KeyCode::KC_VIDEOPHONE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_GAMES) => event::KeyCode::KC_GAMES,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ZOOMIN) => event::KeyCode::KC_ZOOMIN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ZOOMOUT) => event::KeyCode::KC_ZOOMOUT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ZOOMRESET) => event::KeyCode::KC_ZOOMRESET,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_WORDPROCESSOR) => {
            event::KeyCode::KC_WORDPROCESSOR
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_EDITOR) => event::KeyCode::KC_EDITOR,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SPREADSHEET) => event::KeyCode::KC_SPREADSHEET,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_GRAPHICSEDITOR) => {
            event::KeyCode::KC_GRAPHICSEDITOR
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PRESENTATION) => {
            event::KeyCode::KC_PRESENTATION
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DATABASE) => event::KeyCode::KC_DATABASE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NEWS) => event::KeyCode::KC_NEWS,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_VOICEMAIL) => event::KeyCode::KC_VOICEMAIL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ADDRESSBOOK) => event::KeyCode::KC_ADDRESSBOOK,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MESSENGER) => event::KeyCode::KC_MESSENGER,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DISPLAYTOGGLE) => {
            event::KeyCode::KC_DISPLAYTOGGLE
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SPELLCHECK) => event::KeyCode::KC_SPELLCHECK,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LOGOFF) => event::KeyCode::KC_LOGOFF,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DOLLAR) => event::KeyCode::KC_DOLLAR,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_EURO) => event::KeyCode::KC_EURO,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FRAMEBACK) => event::KeyCode::KC_FRAMEBACK,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FRAMEFORWARD) => {
            event::KeyCode::KC_FRAMEFORWARD
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CONTEXT_MENU) => {
            event::KeyCode::KC_CONTEXT_MENU
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MEDIA_REPEAT) => {
            event::KeyCode::KC_MEDIA_REPEAT
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_10CHANNELSUP) => {
            event::KeyCode::KC_10CHANNELSUP
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_10CHANNELSDOWN) => {
            event::KeyCode::KC_10CHANNELSDOWN
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_IMAGES) => event::KeyCode::KC_IMAGES,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DEL_EOL) => event::KeyCode::KC_DEL_EOL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DEL_EOS) => event::KeyCode::KC_DEL_EOS,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_INS_LINE) => event::KeyCode::KC_INS_LINE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DEL_LINE) => event::KeyCode::KC_DEL_LINE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN) => event::KeyCode::KC_FN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_ESC) => event::KeyCode::KC_FN_ESC,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F1) => event::KeyCode::KC_FN_F1,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F2) => event::KeyCode::KC_FN_F2,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F3) => event::KeyCode::KC_FN_F3,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F4) => event::KeyCode::KC_FN_F4,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F5) => event::KeyCode::KC_FN_F5,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F6) => event::KeyCode::KC_FN_F6,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F7) => event::KeyCode::KC_FN_F7,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F8) => event::KeyCode::KC_FN_F8,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F9) => event::KeyCode::KC_FN_F9,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F10) => event::KeyCode::KC_FN_F10,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F11) => event::KeyCode::KC_FN_F11,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F12) => event::KeyCode::KC_FN_F12,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_1) => event::KeyCode::KC_FN_1,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_2) => event::KeyCode::KC_FN_2,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_D) => event::KeyCode::KC_FN_D,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_E) => event::KeyCode::KC_FN_E,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_F) => event::KeyCode::KC_FN_F,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_S) => event::KeyCode::KC_FN_S,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FN_B) => event::KeyCode::KC_FN_B,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRL_DOT1) => event::KeyCode::KC_BRL_DOT1,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRL_DOT2) => event::KeyCode::KC_BRL_DOT2,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRL_DOT3) => event::KeyCode::KC_BRL_DOT3,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRL_DOT4) => event::KeyCode::KC_BRL_DOT4,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRL_DOT5) => event::KeyCode::KC_BRL_DOT5,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRL_DOT6) => event::KeyCode::KC_BRL_DOT6,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRL_DOT7) => event::KeyCode::KC_BRL_DOT7,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRL_DOT8) => event::KeyCode::KC_BRL_DOT8,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRL_DOT9) => event::KeyCode::KC_BRL_DOT9,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRL_DOT10) => event::KeyCode::KC_BRL_DOT10,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_0) => event::KeyCode::KC_NUMERIC_0,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_1) => event::KeyCode::KC_NUMERIC_1,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_2) => event::KeyCode::KC_NUMERIC_2,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_3) => event::KeyCode::KC_NUMERIC_3,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_4) => event::KeyCode::KC_NUMERIC_4,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_5) => event::KeyCode::KC_NUMERIC_5,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_6) => event::KeyCode::KC_NUMERIC_6,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_7) => event::KeyCode::KC_NUMERIC_7,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_8) => event::KeyCode::KC_NUMERIC_8,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_9) => event::KeyCode::KC_NUMERIC_9,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_STAR) => {
            event::KeyCode::KC_NUMERIC_STAR
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_POUND) => {
            event::KeyCode::KC_NUMERIC_POUND
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_A) => event::KeyCode::KC_NUMERIC_A,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_B) => event::KeyCode::KC_NUMERIC_B,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_C) => event::KeyCode::KC_NUMERIC_C,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_D) => event::KeyCode::KC_NUMERIC_D,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CAMERA_FOCUS) => {
            event::KeyCode::KC_CAMERA_FOCUS
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_WPS_BUTTON) => event::KeyCode::KC_WPS_BUTTON,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TOUCHPAD_TOGGLE) => {
            event::KeyCode::KC_TOUCHPAD_TOGGLE
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TOUCHPAD_ON) => event::KeyCode::KC_TOUCHPAD_ON,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TOUCHPAD_OFF) => {
            event::KeyCode::KC_TOUCHPAD_OFF
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CAMERA_ZOOMIN) => {
            event::KeyCode::KC_CAMERA_ZOOMIN
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CAMERA_ZOOMOUT) => {
            event::KeyCode::KC_CAMERA_ZOOMOUT
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CAMERA_UP) => event::KeyCode::KC_CAMERA_UP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CAMERA_DOWN) => event::KeyCode::KC_CAMERA_DOWN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CAMERA_LEFT) => event::KeyCode::KC_CAMERA_LEFT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CAMERA_RIGHT) => {
            event::KeyCode::KC_CAMERA_RIGHT
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ATTENDANT_ON) => {
            event::KeyCode::KC_ATTENDANT_ON
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ATTENDANT_OFF) => {
            event::KeyCode::KC_ATTENDANT_OFF
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ATTENDANT_TOGGLE) => {
            event::KeyCode::KC_ATTENDANT_TOGGLE
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LIGHTS_TOGGLE) => {
            event::KeyCode::KC_LIGHTS_TOGGLE
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ALS_TOGGLE) => event::KeyCode::KC_ALS_TOGGLE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ROTATE_LOCK_TOGGLE) => {
            event::KeyCode::KC_ROTATE_LOCK_TOGGLE
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BUTTONCONFIG) => {
            event::KeyCode::KC_BUTTONCONFIG
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_TASKMANAGER) => event::KeyCode::KC_TASKMANAGER,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_JOURNAL) => event::KeyCode::KC_JOURNAL,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_CONTROLPANEL) => {
            event::KeyCode::KC_CONTROLPANEL
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SCREENSAVER) => event::KeyCode::KC_SCREENSAVER,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_VOICECOMMAND) => {
            event::KeyCode::KC_VOICECOMMAND
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ASSISTANT) => event::KeyCode::KC_ASSISTANT,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRIGHTNESS_MIN) => {
            event::KeyCode::KC_BRIGHTNESS_MIN
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_BRIGHTNESS_MAX) => {
            event::KeyCode::KC_BRIGHTNESS_MAX
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KBDINPUTASSIST_PREV) => {
            event::KeyCode::KC_KBDINPUTASSIST_PREV
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KBDINPUTASSIST_NEXT) => {
            event::KeyCode::KC_KBDINPUTASSIST_NEXT
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KBDINPUTASSIST_PREVGROUP) => {
            event::KeyCode::KC_KBDINPUTASSIST_PREVGROUP
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KBDINPUTASSIST_NEXTGROUP) => {
            event::KeyCode::KC_KBDINPUTASSIST_NEXTGROUP
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KBDINPUTASSIST_ACCEPT) => {
            event::KeyCode::KC_KBDINPUTASSIST_ACCEPT
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_KBDINPUTASSIST_CANCEL) => {
            event::KeyCode::KC_KBDINPUTASSIST_CANCEL
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RIGHT_UP) => event::KeyCode::KC_RIGHT_UP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_RIGHT_DOWN) => event::KeyCode::KC_RIGHT_DOWN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LEFT_UP) => event::KeyCode::KC_LEFT_UP,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_LEFT_DOWN) => event::KeyCode::KC_LEFT_DOWN,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ROOT_MENU) => event::KeyCode::KC_ROOT_MENU,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MEDIA_TOP_MENU) => {
            event::KeyCode::KC_MEDIA_TOP_MENU
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_11) => event::KeyCode::KC_NUMERIC_11,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NUMERIC_12) => event::KeyCode::KC_NUMERIC_12,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_AUDIO_DESC) => event::KeyCode::KC_AUDIO_DESC,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_3D_MODE) => event::KeyCode::KC_3D_MODE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_NEXT_FAVORITE) => {
            event::KeyCode::KC_NEXT_FAVORITE
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_STOP_RECORD) => event::KeyCode::KC_STOP_RECORD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PAUSE_RECORD) => {
            event::KeyCode::KC_PAUSE_RECORD
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_VOD) => event::KeyCode::KC_VOD,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_UNMUTE) => event::KeyCode::KC_UNMUTE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_FASTREVERSE) => event::KeyCode::KC_FASTREVERSE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_SLOWREVERSE) => event::KeyCode::KC_SLOWREVERSE,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_DATA) => event::KeyCode::KC_DATA,
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_ONSCREEN_KEYBOARD) => {
            event::KeyCode::KC_ONSCREEN_KEYBOARD
        }
        enums::EventCode::EV_KEY(enums::EV_KEY::KEY_MAX) => event::KeyCode::KC_MAX,
        _ => event::KeyCode::NotImplemented,
    }
}

fn i32_into_ks(i: i32) -> event::KeyState {
    match i {
        0 => event::KeyState::Up,
        1 => event::KeyState::Down,
        2 => event::KeyState::Held,
        _ => event::KeyState::NotImplemented,
    }
}
