use clap::{App, AppSettings, Arg, SubCommand};
use std::io::{self, Write};

mod argument;

#[tokio::main]
async fn main() {
    let app = argument::get_app();
    
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
