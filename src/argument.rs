use std::{error::Error, str::FromStr};

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub const CREATE: &str = "create";
pub const DELETE: &str = "delete";
pub const LIST: &str = "list";
pub const LS: &str = "ls";
pub const DEFAULT_WORK_TIME: u16 = 25;
pub const DEFAULT_BREAK_TIME: u16 = 5;
pub const TEST: &str = "test";
pub const CLEAR: &str = "clear";

pub fn get_app() -> App<'static, 'static> {
    App::new("pomodoro")
        .setting(AppSettings::NoBinaryName)
        .version("0.0.1")
        .version_short("v")
        .author("Young")
        .about("manage your time!")
        .subcommands(vec![
            SubCommand::with_name(CREATE)
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
                )
                .arg(
                    Arg::with_name("default")
                        .help(
                            "The flag to create default notification, 25 mins work and 5 min break",
                        )
                        .conflicts_with("work")
                        .conflicts_with("break")
                        .short("d")
                        .long("default"),
                ), // TODO(young): add default argument.
            // TODO(young): Check is possible to detect
            // TODO(young): if default arg is specified then other args should not be specified.
            SubCommand::with_name(DELETE)
                .about("delete a notification")
                .arg(
                    Arg::with_name("id")
                        .help("The ID of notification to delete")
                        .takes_value(true)
                        .conflicts_with("all")
                        .short("i")
                        .long("id"),
                )
                .arg(
                    Arg::with_name("all")
                        .help("The flag to delete all notifications")
                        .short("a")
                        .long("all"),
                ),
            SubCommand::with_name(LIST).about("list notifications long command"),
            SubCommand::with_name(LS).about("list notifications short command"),
            SubCommand::with_name(TEST).about("test notification"),
            SubCommand::with_name(CLEAR).about("clear terminal"),
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
