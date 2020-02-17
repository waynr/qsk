use std::error;
use std::fs::File;
use std::path::PathBuf;

use async_std::sync::channel;
use async_std::task;
use async_std::sync::Sender;
use async_std::sync::Receiver;
use async_std::prelude::FutureExt;
use async_std::prelude::StreamExt;
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

trait InputTransformer {
    fn transform(&mut self, ie: &InputEvent) -> Vec<ControlCode>;
}

struct Handler {
    input_transformer: Box<dyn InputTransformer + Send>,
}

impl Handler {
    async fn handle(mut self, mut r: Receiver<InputEvent>, s: Sender<InputEvent>) {
        while let Some(ie) = r.next().await {
            debug!("received InputEvent from input task");
            let iev = self.input_transformer.transform(&ie);
            for cc in iev.iter() {
                match cc {
                    ControlCode::InputEvent(v) => s.send(v.clone()).await,
                    ControlCode::Exit => return,
                }
            }
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
    let (handler_sender, mut output_receiver) = channel(1);

    let handler = Handler{input_transformer: Box::new(Passthrough{})};
    debug!("creating handler task");
    let handler_task = task::Builder::new().name("handler".to_string())
        .spawn(handler.handle(handler_receiver, handler_sender))?;

    debug!("creating input task");
    let input_task = task::Builder::new().name("input".to_string()).spawn(async move {
        loop {
            let t = myd.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING);
            debug!("received InputEvent from keyboard");
            match t {
                Ok(a) => input_sender.send(a.1).await,
                Err(errno) => error!("error reading from keyboard device: {:?}", errno),
            }
            debug!("sent InputEvent to handler");
        }
    })?;

    debug!("creating output task");
    let output_task = task::Builder::new().name("output".to_string()).spawn(async move {
        while let Some(ie) = output_receiver.next().await {
            debug!("received InputEvent from handler");
            match ui.send_key(&ie) {
                Ok(_) => (),
                Err(errno) => error!("error writing to keyboard device: {:?}", errno),
            }
            debug!("sent InputEvent to virtual keyboard");
        }
    })?;

    input_task.race(output_task).race(handler_task).await;

    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    task::block_on(doit())
}
