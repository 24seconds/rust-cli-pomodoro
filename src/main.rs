use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use tokio::sync::mpsc::{self, Sender};
use tokio::task::JoinHandle;
use tokio::time::{sleep, sleep_until, Duration};

mod argument;
mod database;
mod message;
mod notification;
use database as db;

use crate::argument::parse_arg;
use crate::message::Message;
use crate::notification::Notification;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut glue = db::get_memory_glue();

    db::initialize(&mut glue).await;

    let test_input = vec!["create", "-w", "25", "-b", "5"];
    let test_input = vec!["create", "-w=5", "-b", "2"];
    // let test_input = vec!["help", "create"];

    let mut id_manager: u16 = 1;
    let mut hash_map: HashMap<u16, JoinHandle<()>> = HashMap::new();

    let (tx, mut rx) = mpsc::channel::<Message>(64);

    let tx_for_command = tx.clone();
    tokio::spawn(async move {
        let mut t = 0;

        loop {
            println!("inside spawn blocking");
            let command = read_command(t);
            t += 1;
            println!("user input: {}", &command);

            let _ = tx_for_command.send(Message::UserInput { command }).await;
        }
    });

    while let Some(message) = rx.recv().await {
        match message {
            Message::UserInput { command } => {
                let app = argument::get_app();
                let input = command.split_whitespace();
                let matches = app.get_matches_from(input);

                match matches.subcommand() {
                    ("create", Some(sub_matches)) => {
                        let work_time = parse_arg::<u16>(sub_matches, "work")?;
                        let break_time = parse_arg::<u16>(sub_matches, "break")?;

                        let id = get_new_id(&mut id_manager);
                        db::create_notification(
                            &mut glue,
                            &Notification::new(id, work_time, break_time),
                        )
                        .await;

                        let tx = tx.clone();
                        let handle = spawn_notification(tx, id, work_time, break_time);
                        hash_map.insert(id, handle);
                    }
                    _ => {}
                }
            }
            Message::Delete { id } => {
                println!("Message::Delete called! {}", id);

                hash_map
                    .get(&id)
                    .ok_or(format!("failed to corresponding task (id: {})", &id))?
                    .abort();

                hash_map
                    .remove(&id)
                    .ok_or(format!("failed to revmoe id ({})", id))?;

                db::delete_notification(&mut glue, id).await;

                println!("Message::Delete done");
            }
            _ => {
                panic!("no such message type!");
            }
        }
    }

    sleep(Duration::from_secs(30)).await;

    Ok(())
}

fn get_new_id(id_manager: &mut u16) -> u16 {
    let id = id_manager.clone();
    *id_manager += 1;

    return id;
}

fn read_command(t: i32) -> String {
    if t == 0 {
        String::from("create -w=5 -b 2")
    } else {
        print!("> ");
        io::stdout().flush().expect("could not flush stdout");

        let mut command = String::new();

        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");

        let command = command.trim().to_string();

        command
    }
}

fn spawn_notification(
    tx: Sender<Message>,
    id: u16,
    work_time: u16,
    break_time: u16,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        // TODO(geunyeong): Use sleep_until instead of sleep
        // sleep_until(deadline)
        println!("id: {}, task started", id);

        let wt = tokio::time::Duration::from_secs(work_time as u64);
        sleep(wt).await;
        println!("id ({}), work time ({}) done", id, work_time);

        let bt = tokio::time::Duration::from_secs(break_time as u64);
        sleep(bt).await;
        println!("id ({}), break time ({}) done", id, break_time);

        let _ = tx.send(Message::Delete { id }).await;

        println!("id: {}, notification work time done!", id);
    })
}
