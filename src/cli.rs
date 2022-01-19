use std::panic;

use async_std;
use clap::{App, Arg, ArgMatches};
use fern;
use log;

pub fn get_clap_app() -> Result<ArgMatches, Box<dyn std::error::Error>> {
    let remap = App::new("remap")
        .arg(
            Arg::new("passthrough")
                .short('p')
                .long("passthrough")
                .takes_value(false)
                .help("Use passthrough mapper (for debugging purposes)"),
        )
        .arg(
            Arg::new("device-file")
                .help("Input events file")
                .takes_value(true)
                .required(true),
        )
        .about("remap the keyboard represented by the specified device file");

    let list_devices = App::new("list-devices")
        .about("list keyboard-type devices available for remapping");

    let matches = App::new("quantom soft keyboard")
        .arg(
            Arg::new("verbose")
                .short('v')
                .multiple_occurrences(true)
                .help("increases the verbosity level"),
        )
        .arg(Arg::new("quiet").short('q').multiple_occurrences(true).help(
            "decreases the verbosity level; once suppresses warnings, twice suppresses errors.",
        ))
        .version("0.0")
        .author("Wayne Warren <wayne.warren.s@gmail.com>")
        .about("The keyboard remapping software you never knew you wanted.")
        .subcommand(remap)
        .subcommand(list_devices)
        .get_matches();

    let vs = matches.occurrences_of("verbose") as usize;
    let lvl_fltr = match vs {
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        3 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Warn,
    };

    let qs = matches.occurrences_of("quiet") as usize;
    let lvl_fltr = match qs {
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Off,
        _ => lvl_fltr,
    };

    setup_logger_fern(Some(lvl_fltr))?;

    Ok(matches)
}

fn setup_logger_fern(level: Option<log::LevelFilter>) -> Result<(), fern::InitError> {
    let mut dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            let result = panic::catch_unwind(|| async_std::task::current());
            let name = match result {
                Ok(task) => match task.name() {
                    Some(s) => s.to_string(),
                    None => "root".to_string(),
                },
                Err(_) => "root".to_string(),
            };
            out.finish(format_args!("{} {:?}: {}", record.level(), name, message))
        })
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?);

    if let Some(lvl_fltr) = &level {
        dispatch = dispatch.level(*lvl_fltr);
    };

    dispatch.apply()?;

    Ok(())
}
