use std::{error::Error, str::FromStr};

use clap::{Arg, ArgMatches, Command};

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
pub const HISTORY: &str = "history";

pub fn get_config_command() -> Command<'static> {
    // TODO(young): revisit crate_version macro
    Command::new("pomodoro").version("crate_version!()").arg(
        Arg::new("config")
            .help("read credential json file from this path")
            .takes_value(true)
            .short('c'),
    )
}

pub fn get_command() -> Command<'static> {
    Command::new("pomodoro")
        .no_binary_name(true)
        .version(env!("CARGO_PKG_VERSION"))
        .author("Young")
        .about("manage your time!")
        .subcommands(vec![
            {
                let cmd = Command::new(CREATE)
                    .alias("c")
                    .about("create the notification");
                add_args_for_create_subcommand(cmd)
            },
            {
                let cmd = Command::new(QUEUE)
                    .alias(Q)
                    .about("create the notification");
                add_args_for_create_subcommand(cmd)
            },
            Command::new(DELETE)
                .alias("d")
                .about("delete a notification")
                .arg(
                    Arg::new("id")
                        .help("The ID of notification to delete")
                        .takes_value(true)
                        .conflicts_with("all")
                        .short('i'),
                )
                .arg(
                    Arg::new("all")
                        .help("The flag to delete all notifications")
                        .short('a'),
                ),
            Command::new(LIST).alias(LS).about("list notifications"),
            Command::new(HISTORY).about("show archived notifications"),
            Command::new(TEST).about("test notification"),
            Command::new(CLEAR).about("clear terminal"),
            Command::new(EXIT).about("exit pomodoro app"),
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

pub(crate) fn add_args_for_create_subcommand(command: Command<'_>) -> Command {
    let command = command
        .arg(
            Arg::new("work")
                .help("The focus time. Unit is minutes")
                .takes_value(true)
                .short('w')
                .default_value("0"),
        )
        .arg(
            Arg::new("break")
                .help("The break time, Unit is minutes")
                .takes_value(true)
                .short('b')
                .default_value("0"),
        )
        .arg(
            Arg::new("default")
                .help("The flag to create default notification, 25 mins work and 5 min break")
                .conflicts_with("work")
                .conflicts_with("break")
                .short('d')
                .long("default"),
        );

    command
}

#[cfg(test)]
mod tests {
    use clap::{Arg, Command};

    use super::{add_args_for_create_subcommand, get_command, get_config_command, parse_arg};

    #[test]
    fn test_get_command() {
        let app = get_command();
        assert_eq!(app.get_name(), "pomodoro");
    }

    #[test]
    fn test_parse_arg() {
        let m = Command::new("myapp")
            .arg(Arg::new("id").takes_value(true))
            .get_matches_from("myapp abc".split_whitespace());

        // parse as expected
        let id = parse_arg::<String>(&m, "id").unwrap_or_else(|e| panic!("An error occurs: {}", e));
        assert!(id.eq("abc"));

        let m = Command::new("myapp")
            .arg(Arg::new("id").takes_value(true))
            .get_matches_from("myapp abc".split_whitespace());

        // error when parsing
        let id = parse_arg::<u16>(&m, "id");
        assert!(id.is_err());
        assert!(id
            .err()
            .unwrap()
            .to_string()
            .contains("failed to parse arg"));
    }

    #[test]
    fn test_add_args_for_creation() {
        // test work and break
        let cmd = Command::new("myapp");
        let matches = add_args_for_create_subcommand(cmd)
            .get_matches_from("myapp -w 25 -b 5".split_whitespace());

        let work = matches.value_of("work").unwrap();
        assert!(work.eq("25"));
        let r#break = matches.value_of("break").unwrap();
        assert!(r#break.eq("5"));

        // test default
        let cmd = Command::new("myapp");
        let matches =
            add_args_for_create_subcommand(cmd).get_matches_from("myapp -d".split_whitespace());

        assert!(matches.is_present("default"));
    }

    #[test]
    fn test_config_command() {
        let cmd =
            get_config_command().get_matches_from("pomodoro -c credential.json".split_whitespace());
        let config = cmd.value_of("config").unwrap();
        assert_eq!(config, "credential.json");
    }
}
