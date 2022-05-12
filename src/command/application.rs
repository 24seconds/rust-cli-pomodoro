use clap::{Arg, ArgMatches, Command};
use std::sync::Arc;

use crate::command::action::ActionType;
use crate::configuration::Configuration;

pub const CREATE: &str = "create";
pub const QUEUE: &str = "queue";
pub const Q: &str = "q";
pub const DELETE: &str = "delete";
pub const LIST: &str = "list";
pub const LS: &str = "ls";
pub const TEST: &str = "test";
pub const EXIT: &str = "exit";
pub const CLEAR: &str = "clear";
pub const HISTORY: &str = "history";

const AUTHOR: &str = "Young";
const BINARY_NAME: &str = "pomodoro";

pub const DEFAULT_WORK_TIME: u16 = 25;
pub const DEFAULT_BREAK_TIME: u16 = 5;

pub enum CommandType {
    StartUp(Arc<Configuration>),
    UdsClient(ArgMatches),
}

pub fn get_start_and_uds_client_command() -> Command<'static> {
    Command::new(BINARY_NAME)
        .version(env!("CARGO_PKG_VERSION"))
        .author(AUTHOR)
        .about("uds client command")
        .args_conflicts_with_subcommands(true)
        .arg(
            Arg::new("config")
                .help("read credential json file from this path")
                .takes_value(true)
                .value_name("FILE")
                .short('c')
                .long("config"),
        )
        .subcommands(get_common_subcommands())
}

pub fn get_main_command() -> Command<'static> {
    Command::new(BINARY_NAME)
        .no_binary_name(true)
        .version(env!("CARGO_PKG_VERSION"))
        .author(AUTHOR)
        .about("manage your time!")
        .subcommands({
            let mut cmds = get_common_subcommands();
            cmds.push(Command::new(CLEAR).about("clear terminal"));
            cmds.push(Command::new(EXIT).about("exit pomodoro app"));

            cmds
        })
}

fn get_common_subcommands() -> Vec<Command<'static>> {
    vec![
        {
            let cmd = Command::new(ActionType::Create)
                .alias("c")
                .about("create the notification");
            add_args_for_create_subcommand(cmd)
        },
        {
            let cmd = Command::new(ActionType::Queue)
                .alias(Q)
                .about("create the notification");
            add_args_for_create_subcommand(cmd)
        },
        Command::new(ActionType::Delete)
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
        Command::new(ActionType::List)
            .alias(LS)
            .about("list notifications"),
        Command::new(ActionType::History).about("show archived notifications"),
        Command::new(ActionType::Test).about("test notification"),
    ]
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
    use std::iter::zip;

    use clap::{Arg, Command};

    use crate::command::application::get_common_subcommands;

    use super::{
        add_args_for_create_subcommand, get_main_command, get_start_and_uds_client_command, AUTHOR,
        BINARY_NAME,
    };

    #[test]
    fn test_get_start_and_uds_client_command() {
        let command = get_start_and_uds_client_command();

        assert_eq!(command.get_name(), BINARY_NAME);
        assert_eq!(command.get_author().unwrap(), AUTHOR);

        let args: Vec<&Arg> = command
            .get_arguments()
            .filter(|arg| arg.get_id() == "config")
            .collect();
        assert_eq!(args.len(), 1);

        let subcommands = command.get_subcommands().collect::<Vec<&Command>>();
        let common_subcommand = get_common_subcommands();

        zip(subcommands, common_subcommand)
            .for_each(|(actual, expected)| assert_eq!(actual, &expected));
    }

    #[test]
    fn test_get_main_command() {
        let app = get_main_command();
        assert_eq!(app.get_name(), "pomodoro");
    }

    #[test]
    fn test_get_common_subcommands() {
        let subcommands = get_common_subcommands();
        assert!(subcommands.len() == 6);
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
}
