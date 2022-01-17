use futures::executor::block_on;
use std::path::PathBuf;

use evdev;
use evdev::uinput;

use qsk_errors::{Error, Result};
use qsk_events as event;

pub struct Device {
    inner: evdev::EventStream,
}

impl Device {
    pub fn from_path(path: PathBuf) -> Result<Device> {
        let d = evdev::Device::open(&path)?;
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

    pub fn new_uinput_device(&self) -> Result<UInputDevice> {
        let mut vdb = uinput::VirtualDeviceBuilder::new()?;
        vdb = vdb.name("meow");
        if let Some(sks) = self.inner.device().supported_keys() {
            vdb = vdb.with_keys(sks)?;
        } else {
            return Err(Error::NoSupportedKeys);
        }
        Ok(UInputDevice {
            inner: vdb.build()?,
        })
    }
}

impl event::InputEventSource for Device {
    fn recv(&mut self) -> Result<Option<event::InputEvent>> {
        let ev = block_on(self.inner.next_event())?;
        if let Some(ec) = num::FromPrimitive::from_u16(ev.code()) {
            match ev.event_type() {
                evdev::EventType::KEY => Ok(Some(event::InputEvent {
                    time: ev.timestamp(),
                    code: ec,
                    state: i32_into_ks(ev.value()),
                    ty: event::EventType::KEY,
                })),
                evdev::EventType::SYNCHRONIZATION => Ok(Some(event::InputEvent {
                    time: ev.timestamp(),
                    code: ec,
                    state: i32_into_ks(ev.value()),
                    ty: event::EventType::SYN,
                })),
                _ => Ok(None),
            }
        } else {
            Err(Error::UnrecognizedInputEvent)
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

pub struct UInputDevice {
    inner: evdev::uinput::VirtualDevice,
}

impl event::InputEventSink for UInputDevice {
    fn send(&mut self, ke: event::InputEvent) -> Result<()> {
        self.inner.emit(&[evdev::InputEvent::new(
            evdev::EventType::KEY,
            ke.code as u16,
            ke.state as i32,
        )])?;
        Ok(())
    }
}
