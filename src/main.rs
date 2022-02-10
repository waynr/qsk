use std::error;
use std::thread::sleep;
use std::time::Duration;

use async_compat::Compat;
use async_std::prelude::FutureExt;
use async_std::task;
use clap::ArgMatches;
use ctrlc;
use maplit::hashmap;

mod cli;
use cli::get_clap_app;

use qsk_types::control_code::ControlCode;
use qsk::device::linux::Device;
use qsk::device::linux_evdev;
use qsk::engine::QSKEngine;
use qsk::events::KeyCode::*;
use qsk::layers::{
    key, tap_toggle, InputTransformer, Layer, LayerComposer, Passthrough,
};
use qsk::listener::StdoutListener;
use qsk::recorder::Recorder;

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

    // give input source events time to finish before grabbing. this is necessary if the keyboard
    // being remapped is the one where "enter" is pressed on the command line to call `qsk` in the
    // shell
    //
    sleep(Duration::from_millis(300));

    let myd = Device::from_path(input_events_file)?;
    let ui = myd.new_uinput_device()?;

    let mut transformer: Box<dyn InputTransformer + Send>;
    transformer = Box::new(Passthrough {});
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
                    KC_END => vec![ControlCode::Exit],
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
        transformer = Box::new(LayerComposer::from_layers(layers));
    }

    if let Some(path) = matches.value_of("log-keys-to") {
        let (mut recorder, listener) = Recorder::wrap(transformer);
        let engine = QSKEngine::new(Box::new(listener));
        let engine_task = engine.run(Box::new(myd), Box::new(ui));
        let recorder_task = recorder.record(path.into());
        engine_task.race(recorder_task).await?
    } else {
        let engine = QSKEngine::new(transformer);
        engine.run(Box::new(myd), Box::new(ui)).await?;
    }

    Ok(())
}

async fn listen(matches: &ArgMatches) -> Result<(), Box<dyn error::Error>> {
    let input_events_file = matches.value_of_t("device-file")?;
    let myd = Device::from_path(input_events_file)?;
    let mut listener = StdoutListener::from_device(myd);
    listener.listen();

    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = get_clap_app()?;

    match matches.subcommand() {
        Some(("listen", submatches)) => task::block_on(Compat::new(listen(submatches)))?,
        Some(("list-devices", _)) => linux_evdev::Device::list()?,
        Some(("remap", submatches)) => task::block_on(remap(submatches))?,
        _ => (),
    };
    Ok(())
}
