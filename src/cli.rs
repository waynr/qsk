use std::panic;

use async_std;
use clap::{App, Arg, ArgMatches, 
    crate_name,
    crate_authors,
    crate_version,
    crate_description,
};
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
            Arg::new("log-keys-to")
                .long("log-keys-to")
                .takes_value(true)
                .help("Enables keylogging to specified file (for testing purposes)."),
        )
        .arg(
            Arg::new("device-file")
                .help("Input events file")
                .takes_value(true)
                .required(true),
        )
        .about("remap the keyboard represented by the specified device file");

    let listen = App::new("listen")
        .arg(
            Arg::new("device-file")
                .help("Input events file")
                .takes_value(true)
                .required(true),
        )
        .about("listen to and print events stdout");

    let list_devices =
        App::new("list-devices").about("list keyboard-type devices available for remapping");

    let matches = App::new(crate_name!())
        .arg(
            Arg::new("verbose")
                .short('v')
                .multiple_occurrences(true)
                .help("increases the verbosity level"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .multiple_occurrences(true)
                .help(
                "decreases the verbosity level; once suppresses warnings, twice suppresses errors.",
            ),
        )
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(remap)
        .subcommand(listen)
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
