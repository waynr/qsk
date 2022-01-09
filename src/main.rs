use std::error;

use async_std::task;
use maplit::hashmap;

mod cli;
use cli::get_clap_app;

use qsk_device::linux::Device;
use qsk_engine::QSKEngine;
use qsk_events::KeyCode::*;
use qsk_layers::key;
use qsk_layers::tap_toggle;
use qsk_layers::Layer;
use qsk_layers::LayerComposer;

#[derive(Clone, Debug, PartialEq, Copy)]
enum LAYERS {
    HomerowCodeRight = 0,
    Navigation = 1,
}

impl From<LAYERS> for usize {
    fn from(layer: LAYERS) -> usize {
        layer as usize
    }
}

async fn doit() -> Result<(), Box<dyn error::Error>> {
    let matches = get_clap_app()?;
    let input_events_file = matches.value_of_t( "device-file")?;

    let myd = Device::from_path(input_events_file)?;
    let ui = myd.new_uinput_device()?;

    let mut layers = Vec::with_capacity(8);
    layers.insert(
        LAYERS::HomerowCodeRight.into(),
        Layer::from_hashmap(
            hashmap!(
                KC_F => tap_toggle(LAYERS::Navigation.into(), KC_F)
            ),
            true,
        ),
    );
    layers.insert(
        LAYERS::Navigation.into(),
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
            false,
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
