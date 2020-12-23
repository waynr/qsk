use std::error;
use std::path::PathBuf;

use async_std::prelude::FutureExt;
use async_std::prelude::StreamExt;
use async_std::sync::channel;
use async_std::task;
use clap::value_t;
use log::debug;
use log::error;

mod device;
use device::linux::Device;

mod cli;
use cli::get_clap_app;

use qsk_layers::LayerComposer;
use qsk_engine::QSKEngine;


async fn doit() -> Result<(), Box<dyn error::Error>> {
    let matches = get_clap_app()?;
    let input_events_file = value_t!(matches, "device-file", PathBuf)?;

    std::thread::sleep(std::time::Duration::from_millis(1000));
    let myd = Device::from_path(input_events_file)?;
    let ui = myd.new_uinput_device()?;

    let (input_sender, handler_receiver) = channel(1);
    let (handler_sender, mut output_receiver) = channel(1);

    let handler = QSKEngine::new(Box::new(LayerComposer::new()));
    debug!("creating handler task");
    let handler_task = task::Builder::new()
        .name("handler".to_string())
        .spawn(handler.handle(handler_receiver, handler_sender))?;

    debug!("creating input task");
    let input_task = task::Builder::new()
        .name("input".to_string())
        .spawn(async move {
            loop {
                let t = myd.next_event();
                debug!("received KeyboardEvent from keyboard");
                match t {
                    Ok(a) => input_sender.send(a).await,
                    Err(err) => error!("error reading from keyboard device: {:?}", err),
                }
                debug!("sent KeyboardEvent to handler");
            }
        })?;

    debug!("creating output task");
    let output_task = task::Builder::new()
        .name("output".to_string())
        .spawn(async move {
            while let Some(e) = output_receiver.next().await {
                debug!("received KeyboardEvent from handler");
                match ui.send_key(e) {
                    Ok(_) => (),
                    Err(err) => error!("error writing to keyboard device: {:?}", err),
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
