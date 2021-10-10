use clap::{App, AppSettings, Arg, SubCommand};
use std::error::Error;
use std::io::{self, Write};

mod argument;
mod database;
mod notification;
use database as db;

use crate::notification::Notification;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = argument::get_app();

    let mut glue = db::get_memory_glue();

    db::initialize(&mut glue).await;

    let test_input = vec!["create", "-w", "25", "-b", "5"];
    let test_input = vec!["create", "-w=25", "-b", "5"];
    // let test_input = vec!["help", "create"];

    let matches = app.get_matches_from(test_input);

    let mut id_manager: u16 = 1;

    match matches.subcommand() {
        ("create", Some(sub_matches)) => {
            let work = sub_matches
                .value_of("work")
                .ok_or("failed to get work from cli")?;

            let break_str = sub_matches
                .value_of("break")
                .ok_or("failed to get break from cli")?;

            println!("work: {}", work);
            println!("break: {}", break_str);

            let work_time = work.parse::<u16>()?;
            let break_time = break_str.parse::<u16>()?;

            let id = get_new_id(&mut id_manager);
            let creating_notification = Notification::new(id, work_time, break_time);
            db::create_notification(&mut glue, &creating_notification).await;
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
    Ok(())

fn get_new_id(id_manager: &mut u16) -> u16 {
    let id = id_manager.clone();
    *id_manager += 1;

    return id;
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
