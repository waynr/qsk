use std::error;
use std::fs::File;

use evdev_rs::enums;
use evdev_rs::Device;
use evdev_rs::GrabMode;
use evdev_rs::InputEvent;
use evdev_rs::TimeVal;
use evdev_rs::UInputDevice;

struct Handler {
    output_device: UInputDevice,
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
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let f = File::open("/dev/input/by-path/platform-i8042-serio-0-event-kbd")?;

    let mut d = Device::new().unwrap();
    d.set_fd(f)?;
    d.grab(GrabMode::Grab)?;

    let handler = Handler {
        output_device: UInputDevice::create_from_device(&d)?,
    };

    loop {
        let a = d.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING)?;

        match (&a.1.event_type, &a.1.event_code) {
            (enums::EventType::EV_KEY, ec) => match ec {
                enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PAUSE) => break,
                _ => handler.send_key(a.1.time, ec.clone(), a.1.value)?,
            },
            (_, _) => (),
        }
    }
    Ok(())
}
