use std::{error::Error, str::FromStr};

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub const CREATE: &str = "create";
pub const QUEUE: &str = "queue";
// TODO(young): I don't know how to make alias of QUEUE command so I added Q command for aliasing.
pub const Q: &str = "q";
pub const DELETE: &str = "delete";
pub const LIST: &str = "list";
pub const LS: &str = "ls";
pub const DEFAULT_WORK_TIME: u16 = 25;
pub const DEFAULT_BREAK_TIME: u16 = 5;
pub const TEST: &str = "test";
pub const EXIT: &str = "exit";
pub const CLEAR: &str = "clear";

pub fn get_config_app() -> App<'static, 'static> {
    App::new("pomodoro").arg(
        Arg::with_name("config")
            .help("read credential json file from this path")
            .takes_value(true)
            .short("c")
            .long("config"),
    )
}

pub fn get_app() -> App<'static, 'static> {
    App::new("pomodoro")
        .setting(AppSettings::NoBinaryName)
        .version("0.0.1")
        .version_short("v")
        .author("Young")
        .about("manage your time!")
        .subcommands(vec![
            {
                let cmd = SubCommand::with_name(CREATE);
                add_args_for_creation(cmd)
            },
            {
                let cmd = SubCommand::with_name(QUEUE);
                add_args_for_creation(cmd)
            },
            {
                let cmd = SubCommand::with_name(Q);
                add_args_for_creation(cmd)
            },
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
            SubCommand::with_name(EXIT).about("exit pomodoro app"),
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

fn add_args_for_creation<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    let app = app
        .arg(
            Arg::with_name("work")
                .help("The focus time. Unit is minutes")
                .takes_value(true)
                .short("w")
                .long("work")
                .default_value("0"),
        )
        .arg(
            Arg::with_name("break")
                .help("The break time, Unit is minutes")
                .takes_value(true)
                .short("b")
                .long("b")
                .default_value("0"),
        )
        .arg(
            Arg::with_name("default")
                .help("The flag to create default notification, 25 mins work and 5 min break")
                .conflicts_with("work")
                .conflicts_with("break")
                .short("d")
                .long("default"),
        );

    app
}

#[cfg(test)]
mod tests {
    use clap::{App, Arg};

    use super::{get_app, parse_arg};

    #[test]
    fn test_parse_arg() {
        let m = App::new("myapp")
            .arg(Arg::with_name("id").takes_value(true))
            .get_matches_from("myapp abc".split_whitespace());
        
        // parse as expected
        let id = parse_arg::<String>(&m, "id").unwrap_or_else(|e| panic!("An error occurs: {}", e));
        assert!(id.eq("abc"));

        let m = App::new("myapp")
            .arg(Arg::with_name("id").takes_value(true))
            .get_matches_from("myapp abc".split_whitespace());
    
        // error when parsing
        let id = parse_arg::<u16>(&m, "id");
        assert!(id.is_err());
        assert!(id.err().unwrap().to_string().contains("failed to parse arg"));
    }
}
