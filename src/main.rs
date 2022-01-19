use std::error;

use async_std::task;
use async_compat::Compat;
use maplit::hashmap;
use clap::ArgMatches;

mod cli;
use cli::get_clap_app;

use qsk::device::linux;
use qsk::device::linux::Device;
use qsk::engine::QSKEngine;
use qsk::events::KeyCode::*;
use qsk::layers::key;
use qsk::layers::tap_toggle;
use qsk::layers::Layer;
use qsk::layers::LayerComposer;
use qsk::layers::Passthrough;

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

async fn remap(matches: &ArgMatches) -> Result<(), Box<dyn error::Error>> {
    let input_events_file = matches.value_of_t("device-file")?;

    let myd = Device::from_path(input_events_file)?;
    let ui = myd.new_uinput_device()?;

    let mut engine = QSKEngine::new(Box::new(Passthrough{}));
    if !matches.is_present("passthrough") {
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
        engine = QSKEngine::new(Box::new(composer));
    }

    engine.run(Box::new(myd), Box::new(ui)).await?;

    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = get_clap_app()?;

    match matches.subcommand() {
        Some(("list-devices", _)) => linux::Device::list()?,
        Some(("remap", submatches)) => task::block_on(Compat::new(remap(submatches)))?,
        _ => (),
    };
    Ok(())
}
