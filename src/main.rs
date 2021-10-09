use clap::{App, AppSettings, Arg, SubCommand};
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    let app = App::new("pomodoro")
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
        ]);

    let test_input = vec!["create", "-w", "25", "-b", "5"];
    let test_input = vec!["create", "-w=25", "-b", "5"];
    // let test_input = vec!["help", "create"];

    let matches = app.get_matches_from(test_input);

    match matches.subcommand() {
        ("create", Some(sub_matches)) => {
            if let Some(work) = sub_matches.value_of("work") {
                println!("work: {}", work);
            }
            if let Some(break_time) = sub_matches.value_of("break") {
                println!("break: {}", break_time);
            }
        }
        _ => {
            // return error?
        }
    }

    loop {
        println!("try to read command");
        let command = read_command();
        println!("user input: {}", command);
    }
}

fn read_command() -> String {
    print!("> ");
    io::stdout().flush().expect("could not flush stdout");

    let mut command = String::new();

    io::stdin()
        .read_line(&mut command)
        .expect("Failed to read line");

    let command = command.trim().to_string();

    command
}
