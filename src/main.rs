use std::error;
use std::thread::sleep;
use std::time::Duration;

use async_compat::Compat;
use async_std::prelude::FutureExt;
use async_std::task;
use clap::ArgMatches;

mod cli;
use cli::get_clap_app;

use qsk_types::layer_composer::{
    InputTransformer, Passthrough,
};
use qsk_macros;

use qsk::device::linux::Device;
use qsk::device::linux_evdev;
use qsk::engine::QSKEngine;
use qsk::listener::StdoutListener;
use qsk::recorder::Recorder;

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
        let lc = qsk_macros::layer!(
            ModLayer[Active]: {
                Y -> HOME,
                F -> TT(Navigation, F),
            },
            Navigation: {
                END -> Exit(),
                Y -> HOME,
                U -> PAGEDOWN,
                I -> PAGEUP,
                O -> END,
                H -> LEFT,
                J -> DOWN,
                K -> UP,
                SEMICOLON -> RIGHT,
            },
        )?;
        transformer = Box::new(lc);
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
