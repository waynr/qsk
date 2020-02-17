use std::error;
use std::fs::File;
use std::path::PathBuf;

use async_std::sync::channel;
use async_std::task;
use async_std::sync::Sender;
use async_std::sync::Receiver;
use async_std::prelude::FutureExt;
use clap::value_t;
use evdev_rs;
use evdev_rs::enums;
use evdev_rs::GrabMode;
use evdev_rs::InputEvent;
use log::debug;
use log::error;

mod device;
use device::linux::Device;

mod cli;
use cli::get_clap_app;

enum ControlCode {
    InputEvent(InputEvent),
    Exit,
}

struct Passthrough {}

impl InputTransformer for Passthrough {
    fn transform(&mut self, ie: &InputEvent) -> Vec<ControlCode> {
        match &ie.event_code {
            enums::EventCode::EV_KEY(enums::EV_KEY::KEY_PAUSE) => {
                vec![ControlCode::Exit]
            }
            enums::EventCode::EV_KEY(_) => {
                debug!("{:?} {:?}", ie.event_code, ie.value);
                vec![ControlCode::InputEvent(
                    InputEvent {
                        time: ie.time.clone(),
                        event_code: ie.event_code.clone(),
                        event_type: ie.event_type.clone(),
                        value: ie.value.clone(),
                    }
                )]
            }
            _ => { vec![] }
        }
    }
}

struct TediousTransformer {}

impl InputTransformer for TediousTransformer {
    fn transform(&mut self, ie: &InputEvent) -> Vec<ControlCode> {
        vec![ControlCode::InputEvent(
            InputEvent {
                time: ie.time.clone(),
                event_code: ie.event_code.clone(),
                event_type: ie.event_type.clone(),
                value: ie.value.clone(),
            }
        )]
    }
}

trait InputTransformer {
    fn transform(&mut self, ie: &InputEvent) -> Vec<ControlCode>;
}

struct Handler {
    input_transformer: Box<dyn InputTransformer + Send>,
}

impl Handler {
    async fn handle(mut self, r: Receiver<InputEvent>, s: Sender<InputEvent>) {
        loop {
            match r.recv().await {
                Some(ie) => {
                    let iev = self.input_transformer.transform(&ie);
                    for cc in iev.iter() {
                        match cc {
                            ControlCode::InputEvent(v) => s.send(v.clone()).await,
                            ControlCode::Exit => return,
                        }
                    }
                }
                None => return,
            };
        }
    }
}

async fn doit() -> Result<(), Box<dyn error::Error>> {
    let matches = get_clap_app()?;
    let input_events_file = value_t!(matches, "device-file", PathBuf)?;
    let f = File::open(input_events_file)?;

    std::thread::sleep(std::time::Duration::from_millis(1000));
    let mut d = evdev_rs::Device::new().unwrap();
    d.set_fd(f)?;
    d.grab(GrabMode::Grab)?;

    let myd = Device::new(d);
    let ui = myd.new_uinput_device()?;

    let (input_sender, handler_receiver) = channel(1);
    let (handler_sender, output_receiver) = channel(1);

    let handler = Handler{input_transformer: Box::new(Passthrough{})};
    let handler_task = task::spawn(handler.handle(handler_receiver, handler_sender));

    let input_task = task::spawn(async move {
        loop {
            let t = myd.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING);
            match t {
                Ok(a) => input_sender.send(a.1).await,
                Err(errno) => error!("error reading from keyboard device: {:?}", errno),
            }
        }
    });

    let output_task = task::spawn(async move {
        loop {
            let a = match output_receiver.recv().await {
                Some(t) => t,
                None => break,
            };
            match ui.send_key(&a) {
                Ok(_) => (),
                Err(errno) => error!("error writing to keyboard device: {:?}", errno),
            }
        }
    });

    let f = input_task.race(output_task).race(handler_task);
    f.await;

    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    task::block_on(doit())
}
