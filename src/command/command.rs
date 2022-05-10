use std::sync::Arc;
use std::error::Error;

use clap::{Arg, ArgMatches, Command};

use crate::configuration::Configuration;

const CREATE: &str = "create";
const QUEUE: &str = "queue";
const Q: &str = "q";
const DELETE: &str = "delete";
const LIST: &str = "list";
const LS: &str = "ls";
const TEST: &str = "test";
const EXIT: &str = "exit";
const CLEAR: &str = "clear";
const HISTORY: &str = "history";

const AUTHOR: &str = "Young";
const BINARY_NAME: &str = "pomodoro";

pub const DEFAULT_WORK_TIME: u16 = 25;
pub const DEFAULT_BREAK_TIME: u16 = 5;

pub enum ActionType {
    CREATE,
    QUEUE,
    DELETE,
    LIST,
    TEST,
    EXIT,
    CLEAR,
    HISTORY,
}

impl ActionType {
    // TODO(young): handle error
    pub fn parse(s: &str) -> Result<Self, Box<dyn Error>> {
        match s.to_lowercase().as_str() {
            CREATE => Ok(ActionType::CREATE),
            Q | QUEUE => Ok(ActionType::QUEUE),
            DELETE => Ok(ActionType::DELETE),
            LS | LIST => Ok(ActionType::LIST),
            TEST => Ok(ActionType::TEST),
            EXIT => Ok(ActionType::EXIT),
            CLEAR => Ok(ActionType::CLEAR),
            HISTORY => Ok(ActionType::HISTORY),
            _ => Err(Box::from(format!(
                "failed to parse str ({}) to ActionType",
                s
            ))),
        }
    }
}

impl From<ActionType> for String {
    fn from(action: ActionType) -> Self {
        match action {
            ActionType::CREATE => String::from(CREATE),
            ActionType::QUEUE => String::from(QUEUE),
            ActionType::DELETE => String::from(DELETE),
            ActionType::LIST => String::from(LIST),
            ActionType::TEST => String::from(TEST),
            ActionType::EXIT => String::from(EXIT),
            ActionType::CLEAR => String::from(CLEAR),
            ActionType::HISTORY => String::from(HISTORY),
        }
    }
}

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
            let cmd = Command::new(ActionType::CREATE)
                .alias("c")
                .about("create the notification");
            add_args_for_create_subcommand(cmd)
        },
        {
            let cmd = Command::new(ActionType::QUEUE)
                .alias(Q)
                .about("create the notification");
            add_args_for_create_subcommand(cmd)
        },
        Command::new(ActionType::DELETE)
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
        Command::new(ActionType::LIST)
            .alias(LS)
            .about("list notifications"),
        Command::new(ActionType::HISTORY).about("show archived notifications"),
        Command::new(ActionType::TEST).about("test notification"),
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
    use clap::{Arg, Command};

    use super::{add_args_for_create_subcommand, get_config_command, get_main_command};

    #[test]
    fn test_get_command() {
        let app = get_main_command();
        assert_eq!(app.get_name(), "pomodoro");
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
