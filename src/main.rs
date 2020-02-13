use std::error;
use std::fs::File;
use std::path::PathBuf;

use clap::value_t;
use evdev_rs::enums;
use evdev_rs::Device;
use evdev_rs::GrabMode;
use evdev_rs::InputEvent;
use evdev_rs::TimeVal;
use evdev_rs::UInputDevice;
use log::debug;

mod cli;
use cli::get_clap_app;

struct Layer {}

struct Handler {
    output_device: UInputDevice,
    active_layers: Vec<Layer>,
}

enum ControlCode {
    Exit,
}

impl Handler {
    fn send_key(
        &self,
        time: TimeVal,
        ec: enums::EventCode,
        value: i32,
    ) -> Result<(), Box<dyn error::Error>> {
        self.output_device.write_event(&InputEvent {
            time: time.clone(),
            event_type: enums::EventType::EV_KEY,
            event_code: ec,
            value: value,
        })?;
        self.output_device.write_event(&InputEvent {
            time: time.clone(),
            event_type: enums::EventType::EV_SYN,
            event_code: enums::EventCode::EV_SYN(enums::EV_SYN::SYN_REPORT),
            value: 0,
        })?;
        Ok(())
    }

    fn handle(&mut self, ie: &InputEvent) -> Result<Option<ControlCode>, Box<dyn error::Error>> {
        match ie.event_code {
            enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PAUSE) => {
                return Ok(Some(ControlCode::Exit))
            }
            _ => {
                debug!("{:?} {:?}", ie.event_code, ie.value);
                self.send_key(ie.time.clone(), ie.event_code.clone(), ie.value)?;
            }
        }
        Ok(None)
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = get_clap_app()?;
    let input_events_file = value_t!(matches, "device-file", PathBuf)?;
    let f = File::open(input_events_file)?;

    std::thread::sleep(std::time::Duration::from_millis(1000));
    let mut d = Device::new().unwrap();
    d.set_fd(f)?;
    d.grab(GrabMode::Grab)?;

    let mut h = Handler {
        output_device: UInputDevice::create_from_device(&d)?,
        active_layers: Vec::new(),
    };

    loop {
        let a = d.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING)?;

        match &a.1.event_type {
            enums::EventType::EV_KEY => match h.handle(&a.1)? {
                Some(ControlCode::Exit) => break,
                None => (),
            },
            _ => (),
        };
    }

    Ok(())
}
