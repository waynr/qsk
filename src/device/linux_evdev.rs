use futures::executor::block_on;
use std::convert::TryFrom;
use std::path::PathBuf;

use evdev;
use evdev::uinput;

use crate::errors::{Error, Result};
use crate::events;
use crate::events::{EventCode, KeyCode, KeyState, SynCode};
use crate::device::traits::{InputEventSink, InputEventSource};

pub struct InputEvent(events::InputEvent);

pub struct Device {
    inner: evdev::EventStream,
}

impl Device {
    pub fn from_path(path: PathBuf) -> Result<Device> {
        let mut d = evdev::Device::open(&path)?;
        d.grab()?;
        Ok(Device {
            inner: d.into_event_stream()?,
        })
    }

    pub fn from_evdev_device(mut d: evdev::Device) -> Result<Device> {
        d.grab()?;
        Ok(Device {
            inner: d.into_event_stream()?,
        })
    }

    pub fn new_uinput_device(&self, name: String) -> Result<UInputDevice> {
        let mut vdb = uinput::VirtualDeviceBuilder::new()?;
        vdb = vdb.name(&name);
        if let Some(sks) = self.inner.device().supported_keys() {
            vdb = vdb.with_keys(sks)?;
        } else {
            return Err(Error::NoSupportedKeys);
        }
        Ok(UInputDevice {
            inner: vdb.build()?,
        })
    }

    pub fn list() -> Result<()> {
        for (path, dev) in evdev::enumerate() {
            if let Some(keys) = dev.supported_keys() {
                let mut key_count = 0;
                for _key in keys.iter() {
                    //println!("  key: {:?}", key);
                    key_count += 1;
                }
                if key_count > 100 {
                    println!("{}", dev.name().unwrap_or("unknown"));
                    println!("  key_count: {}", key_count);
                    println!(
                        "  physical path: {}",
                        dev.physical_path().unwrap_or("unknown")
                    );
                    println!("  system path: {}", path.display());
                }
            }
        }
        Ok(())
    }
}

impl TryFrom<evdev::InputEvent> for InputEvent {
    type Error = Error;

    fn try_from(ev: evdev::InputEvent) -> Result<InputEvent> {
        let ec = match ev.event_type() {
            evdev::EventType::KEY => {
                let kc: Option<KeyCode> = num::FromPrimitive::from_u16(ev.code() as u16);
                match kc {
                    Some(code) => Some(EventCode::KeyCode(code)),
                    None => None,
                }
            }
            evdev::EventType::SYNCHRONIZATION => {
                let kc: Option<SynCode> = num::FromPrimitive::from_u16(ev.code() as u16);
                match kc {
                    Some(code) => Some(EventCode::SynCode(code)),
                    None => None,
                }
            }
            _ => None,
        };
        match ec {
            Some(code) => Ok(InputEvent( events::InputEvent {
                time: ev.timestamp(),
                code,
                state: i32_into_ks(ev.value()),
            })),
            None => Err(Error::UnrecognizedEvdevInputEvent { e: ev }),
        }
    }
}

impl TryFrom<InputEvent> for evdev::InputEvent {
    type Error = Error;

    fn try_from(ie: InputEvent) -> Result<evdev::InputEvent> {
        let (ty, code) = match ie.0.code {
            EventCode::KeyCode(c) => (evdev::EventType::SYNCHRONIZATION, c as i16),
            EventCode::SynCode(c) => (evdev::EventType::KEY, c as i16),
        };
        Ok(evdev::InputEvent::new(ty, code as u16, ie.0.state as i32))
    }
}

impl InputEventSource for Device {
    fn recv(&mut self) -> Result<events::InputEvent> {
        let ev = block_on(self.inner.next_event())?;
        Ok(InputEvent::try_from(ev)?.0)
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

pub struct UInputDevice {
    inner: evdev::uinput::VirtualDevice,
}

impl InputEventSink for UInputDevice {
    fn send(&mut self, ie: events::InputEvent) -> Result<()> {
        let evdev_ie = evdev::InputEvent::try_from(InputEvent(ie))?;
        self.inner.emit(&[evdev_ie])?;
        Ok(())
    }
}
