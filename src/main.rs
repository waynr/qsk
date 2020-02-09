use evdev_rs::enums;
use evdev_rs::Device;
use std::error;
use std::fs::File;

fn main() -> Result<(), Box<dyn error::Error>> {
    let f = File::open("/dev/input/by-path/platform-i8042-serio-0-event-kbd")?;

    let mut d = Device::new().unwrap();
    d.set_fd(f)?;

    loop {
        let a = d.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING)?;

        match (&a.1.event_type, &a.1.event_code) {
            (enums::EventType::EV_KEY, ec) => match ec {
                enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PAUSE) => break,
                _ => println!("{} {}", ec, &a.1.value),
            },
            (_, _) => (),
        }
    }
    Ok(())
}
