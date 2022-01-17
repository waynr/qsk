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
use evdev_rs::InputEvent;
use evdev_rs::TimeVal;
use log::error;

use qsk_errors::Error;
use qsk_errors::Result;

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
    fn recv(&mut self) -> Result<Option<event::InputEvent>> {
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
                match ev.1.event_type {
                    evdev_rs::enums::EventType::EV_KEY => Ok(ie_into_ke(ev.1)),
                    _ => Ok(None),
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

        if let Some(ie) = ke_into_ie(e) {
            match guard.write_event(&ie) {
                Ok(_) => (),
                Err(e) => return Err(Error::IO(e)),
            };
            let t: TimeVal;
            match TimeVal::try_from(e.time) {
                Ok(tv) => t = tv,
                Err(e) => return Err(Error::SystemTimeError(e)),
            }
            match guard.write_event(&InputEvent {
                time: t,
                event_type: enums::EventType::EV_SYN,
                event_code: enums::EventCode::EV_SYN(enums::EV_SYN::SYN_REPORT),
                value: 0,
            }) {
                Ok(_) => return Ok(()),
                Err(e) => return Err(Error::IO(e)),
            }
        }
        Ok(())
    }
}

fn ke_into_ie(kv: event::InputEvent) -> Option<evdev_rs::InputEvent> {
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

fn ie_into_ke(ev: evdev_rs::InputEvent) -> Option<event::InputEvent> {
    match ec_into_kc(ev.event_code) {
        Some(code) => {
            Some(event::InputEvent {
                time: UNIX_EPOCH
                    + Duration::new(ev.time.tv_sec as u64, ev.time.tv_usec as u32 * 1000 as u32),
                code,
                state: i32_into_ks(ev.value),
                ty: num::FromPrimitive::from_u16(ev.event_type as u16).unwrap(),
            })
        }
        None => None,
    }
}

fn kc_into_ec(k: event::KeyCode) -> Option<enums::EventCode> {
    match enums::int_to_ev_key(k as u32) {
        Some(ev_key) => Some(enums::EventCode::EV_KEY(ev_key)),
        None => None,
    }
}

fn ec_into_kc(ec: enums::EventCode) -> Option<event::KeyCode> {
    match ec {
        enums::EventCode::EV_KEY(ec) => num::FromPrimitive::from_u16(ec as u16),
        _ => None,
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
