use std::convert::TryFrom;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use evdev_rs;
use evdev_rs::enums;
use evdev_rs::GrabMode;
use evdev_rs::TimeVal;
use log::error;

use qsk_errors::Error;
use qsk_errors::Result;

use qsk_events as event;
use qsk_events::{EventCode, SynCode, KeyCode};

pub struct Device {
    inner: Arc<Mutex<evdev_rs::Device>>,
}

unsafe impl Send for Device {}

struct InputEvent(event::InputEvent);

impl TryFrom<evdev_rs::InputEvent> for InputEvent {
    type Error = Error;

    fn try_from(ev: evdev_rs::InputEvent) -> Result<InputEvent> {
        let c = match ev.event_code {
            enums::EventCode::EV_KEY(ec) => {
                let kc: Option<KeyCode> = num::FromPrimitive::from_u16(ec as u16);
                match kc {
                    Some(code) => Some(EventCode::KeyCode(code)),
                    None => None,
                }
            },
            enums::EventCode::EV_SYN(ec) => {
                let sc: Option<SynCode> = num::FromPrimitive::from_u16(ec as u16);
                match sc {
                    Some(code) => Some(EventCode::SynCode(code)),
                    None => None,
                }
            },
            _ => None,
        };
        match c {
            Some(code) => {
                Ok(InputEvent(event::InputEvent {
                    time: UNIX_EPOCH
                        + Duration::new(ev.time.tv_sec as u64, ev.time.tv_usec as u32 * 1000 as u32),
                    code,
                    state: i32_into_ks(ev.value),
                }))
            },
            None => {
                Err(Error::UnrecognizedEventCode)
            }
        }
    }
}

impl TryFrom<InputEvent> for evdev_rs::InputEvent {
    type Error = Error;

    fn try_from(ev: InputEvent) -> Result<evdev_rs::InputEvent> {
        let c = match ev.0.code {
            EventCode::KeyCode(c) => {
                match enums::int_to_ev_key(c as u32) {
                    Some(key) => Some(enums::EventCode::EV_KEY(key)),
                    None => None,
                }
            }
            EventCode::SynCode(c) => {
                match enums::int_to_ev_syn(c as u32) {
                    Some(key) => Some(enums::EventCode::EV_SYN(key)),
                    None => None,
                }
            }

        };

        let d = match ev.0.time.duration_since(UNIX_EPOCH) {
            Ok(n) => n,
            Err(_) => Duration::new(0, 0),
        };
        match c {
            Some(event_code) => Ok(evdev_rs::InputEvent {
                time: TimeVal {
                    tv_sec: d.as_secs() as i64,
                    tv_usec: d.subsec_micros() as i64,
                },
                event_type: enums::EventType::EV_KEY,
                event_code,
                value: ev.0.state as i32,
            }),
            None => Err(Error::UnrecognizedEventCode),
        }
    }
}

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
    fn recv(&mut self) -> Result<event::InputEvent> {
        let guard = match self.inner.lock() {
            Ok(a) => a,
            Err(p_err) => {
                let g = p_err.into_inner();
                error!("recovered Device");
                g
            }
        };
        match guard.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING) {
            Ok(ev) => {
                match InputEvent::try_from(ev.1) {
                    Ok(ie) => Ok(ie.0),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(Error::IO(e)),
        }
    }
}

pub struct UInputDevice {
    inner: Arc<Mutex<evdev_rs::UInputDevice>>,
}

unsafe impl Send for UInputDevice {}

impl event::InputEventSink for UInputDevice {
    fn send(&mut self, e: event::InputEvent) -> Result<()> {
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
            event_type: enums::EventType::EV_SYN,
            event_code: enums::EventCode::EV_SYN(enums::EV_SYN::SYN_REPORT),
            value: 0,
        }) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(Error::IO(e)),
        }
    }
}

fn i32_into_ks(i: i32) -> event::KeyState {
    match i {
        1 => event::KeyState::Up,
        0 => event::KeyState::Down,
        2 => event::KeyState::Held,
        _ => event::KeyState::NotImplemented,
    }
}
