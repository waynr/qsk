use std::convert::TryFrom;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use evdev_rs;
use evdev_rs::enums;
use evdev_rs::GrabMode;
use evdev_rs::TimeVal;
use log::error;

use crate::errors::Error;
use crate::errors::Result;

use crate::events;
use crate::events::{EventCode, KeyCode, KeyState, SynCode};
use crate::device::traits::{InputEventSink, InputEventSource};

pub struct InputEvent(events::InputEvent);

pub struct Device {
    inner: Arc<Mutex<evdev_rs::Device>>,
}

unsafe impl Send for Device {}

impl TryFrom<evdev_rs::InputEvent> for InputEvent {
    type Error = Error;

    fn try_from(ev: evdev_rs::InputEvent) -> Result<InputEvent> {
        let c = match ev.event_code {
            enums::EventCode::EV_KEY(ref ec) => {
                let kc: Option<KeyCode> = num::FromPrimitive::from_u16(ec.clone() as u16);
                match kc {
                    Some(code) => Some(EventCode::KeyCode(code)),
                    None => None,
                }
            }
            enums::EventCode::EV_SYN(ref ec) => {
                let sc: Option<SynCode> = num::FromPrimitive::from_u16(ec.clone() as u16);
                match sc {
                    Some(code) => Some(EventCode::SynCode(code)),
                    None => None,
                }
            }
            _ => None,
        };
        match c {
            Some(code) => Ok(InputEvent(events::InputEvent {
                time: UNIX_EPOCH
                    + Duration::new(ev.time.tv_sec as u64, ev.time.tv_usec as u32 * 1000 as u32),
                code,
                state: i32_into_ks(ev.value),
            })),
            None => Err(Error::UnrecognizedEvdevRSInputEvent { e: ev }),
        }
    }
}

impl TryFrom<InputEvent> for evdev_rs::InputEvent {
    type Error = Error;

    fn try_from(ie: InputEvent) -> Result<evdev_rs::InputEvent> {
        let c = match ie.0.code {
            EventCode::KeyCode(c) => match enums::int_to_ev_key(c as u32) {
                Some(key) => Some(enums::EventCode::EV_KEY(key)),
                None => None,
            },
            EventCode::SynCode(c) => match enums::int_to_ev_syn(c as u32) {
                Some(key) => Some(enums::EventCode::EV_SYN(key)),
                None => None,
            },
        };

        let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        match c {
            Some(event_code) => Ok(evdev_rs::InputEvent {
                time: TimeVal {
                    tv_sec: d.as_secs() as i64,
                    tv_usec: d.subsec_micros() as i64,
                },
                event_code,
                value: ie.0.state as i32,
            }),
            None => Err(Error::UnrecognizedInputEvent { e: ie.0 }),
        }
    }
}

impl Device {
    pub fn from_path(path: PathBuf) -> Result<Device> {
        let f = File::open(path)?;
        let mut d = evdev_rs::Device::new_from_file(f).unwrap();
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

impl InputEventSource for Device {
    fn recv(&mut self) -> Result<events::InputEvent> {
        let guard = match self.inner.lock() {
            Ok(a) => a,
            Err(p_err) => {
                let g = p_err.into_inner();
                error!("recovered Device");
                g
            }
        };
        match guard.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING) {
            Ok(ev) => match InputEvent::try_from(ev.1) {
                Ok(ie) => Ok(ie.0),
                Err(e) => Err(e),
            },
            Err(e) => Err(Error::IO(e)),
        }
    }
}

pub struct UInputDevice {
    inner: Arc<Mutex<evdev_rs::UInputDevice>>,
}

unsafe impl Send for UInputDevice {}

impl InputEventSink for UInputDevice {
    fn send(&mut self, e: events::InputEvent) -> Result<()> {
        let guard = match self.inner.lock() {
            Ok(a) => a,
            Err(p_err) => {
                let g = p_err.into_inner();
                error!("recovered Device");
                g
            }
        };

        let ie = evdev_rs::InputEvent::try_from(InputEvent(e))?;
        match guard.write_event(&ie) {
            Ok(_) => (),
            Err(e) => return Err(Error::IO(e)),
        };
        let t: TimeVal;
        match TimeVal::try_from(e.time) {
            Ok(tv) => t = tv,
            Err(e) => return Err(Error::SystemTimeError(e)),
        }
        match guard.write_event(&evdev_rs::InputEvent {
            time: t,
            event_code: enums::EventCode::EV_SYN(enums::EV_SYN::SYN_REPORT),
            value: 0,
        }) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(Error::IO(e)),
        }
    }
}

fn i32_into_ks(i: i32) -> KeyState {
    match i {
        0 => KeyState::Up,
        1 => KeyState::Down,
        2 => KeyState::Held,
        _ => KeyState::NotImplemented,
    }
}
