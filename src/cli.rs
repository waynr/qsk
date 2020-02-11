use clap::{App, Arg, ArgMatches};

use fern;
use log;

pub fn get_clap_app<'a>() -> Result<ArgMatches<'a>, Box<dyn std::error::Error>> {
    let matches = App::new("personal financial analysis tool")
        .arg(
            Arg::with_name("device-file")
                .help("Input events file")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("increases the verbosity level"),
        )
        .arg(Arg::with_name("quiet").short("q").multiple(true).help(
            "decreases the verbosity level; once suppresses warnings, twice suppresses errors.",
        ))
        .version("0.0")
        .author("Wayne Warren <wayne.warren.s@gmail.com>")
        .about("Extrapolative modeling, comparative analysis, and planning for personal finances")
        .to_owned()
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
            out.finish(format_args!(
                "[{}][{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?);

    if let Some(lvl_fltr) = &level {
        dispatch = dispatch.level(*lvl_fltr);
    };

    dispatch.apply()?;

    Ok(())
}
