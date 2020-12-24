use std::error;
use std::path::PathBuf;

use async_std::task;
use clap::value_t;
use maplit::hashmap;

mod device;
use device::linux::Device;

mod cli;
use cli::get_clap_app;

use qsk_layers::LayerComposer;
use qsk_layers::Layer;
use qsk_layers::key;
use qsk_layers::tap_toggle;
use qsk_engine::QSKEngine;
use qsk_events::KeyCode::*;

#[derive(Clone, Debug, PartialEq, Copy)]
enum LAYERS {
    HomerowCodeRight = 0,
    Navigation = 1,
}

impl LAYERS {
    fn to_usize(self) -> usize {
        self as usize
    }
}

async fn doit() -> Result<(), Box<dyn error::Error>> {
    let matches = get_clap_app()?;
    let input_events_file = value_t!(matches, "device-file", PathBuf)?;

    let myd = Device::from_path(input_events_file)?;
    let ui = myd.new_uinput_device()?;

    let mut layers = Vec::with_capacity(8);
    layers.insert(
        LAYERS::HomerowCodeRight.to_usize(),
        Layer::from_hashmap(
            hashmap!(
                KC_F => tap_toggle(LAYERS::Navigation.to_usize(), KC_F)
            ),
            true,
        ),
    );
    layers.insert(
        LAYERS::Navigation.to_usize(),
        Layer::from_hashmap(
            hashmap!(
                KC_Y => key(KC_HOME),
                KC_U => key(KC_PAGEDOWN),
                KC_I => key(KC_PAGEUP),
                KC_O => key(KC_END),
                KC_H => key(KC_LEFT),
                KC_J => key(KC_DOWN),
                KC_K => key(KC_UP),
                KC_SEMICOLON => key(KC_RIGHT),
            ),
            true,
        ),
    );

    let composer = LayerComposer::from_layers(layers);
    let engine = QSKEngine::new(Box::new(composer));

    engine.run(Box::new(myd), Box::new(ui)).await?;

    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    task::block_on(doit())
}
