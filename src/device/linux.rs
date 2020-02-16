use std::sync::Arc;
use std::sync::Mutex;

use evdev_rs;
use evdev_rs::enums;
use evdev_rs::InputEvent;
use evdev_rs::ReadStatus;
use log::error;
use nix::errno::Errno;

pub struct Device {
    inner: Arc<Mutex<evdev_rs::Device>>,
}

unsafe impl Send for Device {}

impl Device {
    pub fn new(device: evdev_rs::Device) -> Device {
        Device {
            inner: Arc::new(Mutex::new(device)),
        }
    }

    pub fn next_event(&self, flags: evdev_rs::ReadFlag) -> Result<(ReadStatus, InputEvent), Errno> {
        let guard = match self.inner.lock() {
            Ok(a) => a,
            Err(p_err) => {
                let g = p_err.into_inner();
                error!("recovered Device");
                g
            }
        };
        guard.next_event(flags)
    }

    pub fn new_uinput_device(&self) -> Result<UInputDevice, Errno> {
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

pub struct UInputDevice {
    inner: Arc<Mutex<evdev_rs::UInputDevice>>,
}

unsafe impl Send for UInputDevice {}

impl UInputDevice {
    pub fn send_key(&self, ie: &InputEvent) -> Result<(), Errno> {
        let guard = match self.inner.lock() {
            Ok(a) => a,
            Err(p_err) => {
                let g = p_err.into_inner();
                error!("recovered Device");
                g
            }
        };

        guard.write_event(ie)?;
        guard.write_event(&InputEvent {
            time: ie.time.clone(),
            event_type: enums::EventType::EV_SYN,
            event_code: enums::EventCode::EV_SYN(enums::EV_SYN::SYN_REPORT),
            value: 0,
        })?;
        Ok(())
    }
}
