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
                .help("Read configuration json file from this path")
                .num_args(1)
                .short('c')
                .long("config"),
        )
        .subcommands({
            let mut cmd = get_common_subcommands();
            cmd.push(
                Command::new("completion")
                    .about("generate completions for shells")
                    .arg(Arg::new("shell").value_parser([
                        "fish",
                        "zsh",
                        "bash",
                        "elvish",
                        "powershell",
                    ])),
            );
            cmd
        })
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
                    .num_args(1)
                    .conflicts_with("all")
                    .short('i')
                    .long("id"),
            )
            .arg(
                Arg::new("all")
                    .help("The flag to delete all notifications")
                    .short('a')
                    .num_args(0)
                    .long("all"),
            ),
        Command::new(ActionType::List)
            .alias(LS)
            .about("list notifications")
            .arg(
                Arg::new("percentage")
                    .short('p')
                    .help("show work time completion percentage")
                    .num_args(0),
            ),
        Command::new(ActionType::History)
            .about("show archived notifications")
            .arg(
                Arg::new("clear")
                    .help("The flag to delete all notifications from history")
                    .short('c')
                    .num_args(0)
                    .long("clear"),
            ),
        Command::new(ActionType::Test).about("test notification"),
    ]
}

pub(crate) fn add_args_for_create_subcommand(command: Command) -> Command {
    command
        .arg(
            Arg::new("work")
                .long_help("The focus time in minutes.
If no value is passed, the work time is obtained from `work_time_default_value` in the given configuration file.
And if no configuration file is passed or `work_time_default_value` is not present, then 25 is used as the work time.
")
                .num_args(1)
                .short('w'),
        )
        .arg(
            Arg::new("break")
                .long_help("The break time in minutes.
If no value is passed, the break time is obtained from `break_time_default_value` in the given configuration file.
And if no configuration file is passed or `break_time_default_value` is not present, then 5 is used as the break time.
")
                .num_args(1)
                .short('b'),
        )
        .arg(
            Arg::new("default")
                .long_help(
                    "The flag to create default notification.
The values are obtained from `work_time_default_value` and `break_time_default_value` in the given configuration file.
If no configuration file is passed or the keys are not present, then 25 minutes for work and 5 minutes for break is considered.
")
                .conflicts_with("work")
                .conflicts_with("break")
                .short('d')
                .long("default")
                .num_args(0),
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
        let completion_cmd = Command::new("completion")
            .about("generate completions for shells")
            .arg(Arg::new("shell").value_parser(["fish", "zsh", "bash", "elvish", "powershell"]));

        let uds_sub_cmds = uds_cmd.get_subcommands().collect::<Vec<&Command>>();
        let mut main_sub_cmds = get_common_subcommands();
        main_sub_cmds.push(completion_cmd);

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
    fn test_delete_command() {
        let cmd = get_main_command();
        let matches = cmd.try_get_matches_from("d -a".split_whitespace());
        assert!(matches.is_ok());
    }

    #[test]
    fn test_get_common_subcommands() {
        let subcommands = get_common_subcommands();
        assert_eq!(subcommands.len(), 6);
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

        assert!(matches.get_flag("default"));

        // test work only
        let cmd = Command::new("myapp");
        let matches =
            add_args_for_create_subcommand(cmd).get_matches_from("myapp -w 25".split_whitespace());
        let work = matches.get_one::<String>("work").unwrap();
        assert!(work.eq("25"));
        assert!(matches.get_one::<String>("break").is_none());
        assert_eq!(matches.get_flag("default"), false);
        assert!(matches.contains_id("default"));

        // test break only
        let cmd = Command::new("myapp");
        let matches =
            add_args_for_create_subcommand(cmd).get_matches_from("myapp -b 10".split_whitespace());
        let break_time = matches.get_one::<String>("break").unwrap();
        assert!(break_time.eq("10"));
        assert!(matches.get_one::<String>("work").is_none());
        assert_eq!(matches.get_flag("default"), false);
        assert!(matches.contains_id("default"));
    }
}
