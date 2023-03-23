use clap::{Arg, ArgAction, ArgMatches, Command};
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
    AutoComplete(ArgMatches),
}

pub fn get_start_and_uds_client_command() -> Command {
    Command::new(BINARY_NAME)
        .version(env!("CARGO_PKG_VERSION"))
        .author(AUTHOR)
        .about("start up application with config or run command using uds client")
        .args_conflicts_with_subcommands(true)
        .arg(
            Arg::new("config")
                .help("read credential json file from this path")
                .value_name("FILE")
                .short('c')
                .long("config"),
        )
        .subcommands(get_common_subcommands())
}

pub fn get_main_command() -> Command {
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

fn get_common_subcommands() -> Vec<Command> {
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
                    .value_name("ID")
                    .conflicts_with("all")
                    .short('i')
                    .long("id"),
            )
            .arg(
                Arg::new("all")
                    .help("The flag to delete all notifications")
                    .short('a')
                    .long("all"),
            ),
        Command::new(ActionType::List)
            .alias(LS)
            .about("list notifications"),
        Command::new(ActionType::History).about("show archived notifications"),
        Command::new(ActionType::Test).about("test notification"),
        Command::new("completion")
            .about("generate completions for shells")
            .arg(Arg::new("shell").value_parser(["fish", "zsh", "bash", "elvish", "powershell"])),
    ]
}

pub(crate) fn add_args_for_create_subcommand(command: Command) -> Command {
    command
        .arg(
            Arg::new("work")
                .help("The focus time. Unit is minutes")
                .value_name("WORK TIME")
                .short('w')
                .default_value("0"),
        )
        .arg(
            Arg::new("break")
                .help("The break time, Unit is minutes")
                .value_name("BREAK TIME")
                .short('b')
                .default_value("0"),
        )
        .arg(
            Arg::new("default")
                .help("The flag to create default notification, 25 mins work and 5 min break")
                .conflicts_with("work")
                .conflicts_with("break")
                .short('d')
                .long("default")
                .action(ArgAction::SetTrue),
        )
}

#[cfg(test)]
mod tests {
    use super::{get_start_and_uds_client_command, AUTHOR, BINARY_NAME};
    use clap::{Arg, Command};

    use crate::command::application::get_common_subcommands;

    use super::{add_args_for_create_subcommand, get_main_command};

    #[test]
    fn test_get_start_and_uds_client_command() {
        let uds_cmd = get_start_and_uds_client_command();

        let uds_sub_cmds = uds_cmd.get_subcommands().collect::<Vec<&Command>>();
        let main_sub_cmds = get_common_subcommands();

        assert_eq!(uds_cmd.get_name(), BINARY_NAME);
        assert_eq!(uds_cmd.get_author().unwrap(), AUTHOR);

        // Test that the number of subcommands is the same
        assert_eq!(main_sub_cmds.len(), uds_sub_cmds.len());

        for (i, main_subcommand) in main_sub_cmds.iter().enumerate() {
            let uds_subcommand = &uds_sub_cmds[i];

            // Test that the subcommand names are the same
            assert_eq!(main_subcommand.get_name(), uds_subcommand.get_name());

            let main_args = main_subcommand.get_arguments().collect::<Vec<&Arg>>();
            let uds_args = uds_subcommand.get_arguments().collect::<Vec<&Arg>>();

            // Test that the number of arguments is the same
            assert_eq!(main_args.len(), uds_args.len());

            for (j, main_arg) in main_args.iter().enumerate() {
                let uds_arg = &uds_args[j];

                // Test that the argument names are the same
                assert_eq!(main_arg.get_id(), uds_arg.get_id());

                // Test that the argument help messages are the same
                assert_eq!(main_arg.get_help(), uds_arg.get_help());

                // Test that the argument short and long names are the same
                assert_eq!(main_arg.get_short(), uds_arg.get_short());
                assert_eq!(main_arg.get_long(), uds_arg.get_long());

                // Test that the argument value names are the same
                assert_eq!(main_arg.get_value_names(), uds_arg.get_value_names());
            }
        }
    }

    #[test]
    fn test_get_main_command() {
        let app = get_main_command();
        assert_eq!(app.get_name(), "pomodoro");
    }

    #[test]
    fn test_get_common_subcommands() {
        let subcommands = get_common_subcommands();
        assert_eq!(subcommands.len(), 7);
    }

    #[test]
    fn test_add_args_for_creation() {
        // test work and break
        let cmd = Command::new("myapp");
        let matches = add_args_for_create_subcommand(cmd)
            .get_matches_from("myapp -w 25 -b 5".split_whitespace());

        let work = matches.get_one::<String>("work").unwrap();
        assert!(work.eq("25"));
        let r#break = matches.get_one::<String>("break").unwrap();
        assert!(r#break.eq("5"));

        // test default
        let cmd = Command::new("myapp");
        let matches =
            add_args_for_create_subcommand(cmd).get_matches_from("myapp -d".split_whitespace());

        assert!(matches.contains_id("default"));
    }
}
