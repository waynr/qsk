use std::error;
use std::path::PathBuf;

use async_std::task;
use clap::value_t;

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

    let handler = QSKEngine::new(Box::new(LayerComposer::new()));
    handler.run(Box::new(myd), Box::new(ui)).await?;
    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    task::block_on(doit())
}
