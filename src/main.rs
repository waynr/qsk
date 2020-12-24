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

    let myd = Device::from_path(input_events_file)?;
    let ui = myd.new_uinput_device()?;

    let engine = QSKEngine::new(Box::new(LayerComposer::new()));
    engine.run(Box::new(myd), Box::new(ui)).await?;
    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    task::block_on(doit())
}
