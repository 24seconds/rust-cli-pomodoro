use std::{error::Error, str::FromStr};

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub fn get_app() -> App<'static, 'static> {
    App::new("pomodoro")
        .setting(AppSettings::NoBinaryName)
        .version("0.0.1")
        .author("Young")
        .about("manage your time!")
        .subcommands(vec![
            SubCommand::with_name("create")
                .about("create a notification")
                .arg(
                    Arg::with_name("work")
                        .help("The focus time. Unit is minutes")
                        .takes_value(true)
                        .short("w")
                        .long("work"),
                )
                .arg(
                    Arg::with_name("break")
                        .help("The break time, Unit is minutes")
                        .takes_value(true)
                        .short("b")
                        .long("b"),
                ), // TODO(young): add default argument.
                   // TODO(young): Check is possible to detect
                   // TODO(young): if default arg is specified then other args should not be specified.
        ])
}

pub fn parse_arg<C>(arg_matches: &ArgMatches, arg_name: &str) -> Result<C, Box<dyn Error>>
where
    C: FromStr,
{
    let str = arg_matches
        .value_of(arg_name)
        .ok_or(format!("failed to get ({}) from cli", arg_name))?;

    let parsed = str
        .parse::<C>()
        .map_err(|_| format!("failed to parse arg ({})", str))?;

    Ok(parsed)
}
