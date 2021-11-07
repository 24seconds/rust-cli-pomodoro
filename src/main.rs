use gluesql::{memory_storage::Key, Glue, MemoryStorage};
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::process;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::sleep;

mod argument;
mod database;
mod message;
mod notification;
use database as db;
mod configuration;

use crate::argument::{
    parse_arg, CLEAR, CREATE, DEFAULT_BREAK_TIME, DEFAULT_WORK_TIME, DELETE, EXIT, LIST, LS, TEST,
};
use crate::configuration::{intialize_configuration, Configuration};
use crate::message::Message;
use crate::notification::{notify_break, notify_work, Notification};

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    initialize_logging();

    let configuration = intialize_configuration()?;
    let configuration = Arc::new(configuration);

    info!("info test, start pomodoro...");
    debug!("debug test, start pomodoro...");

    let mut glue = db::get_memory_glue();

    db::initialize(&mut glue).await;

    tokio::spawn(async {
        let mut test_glue = db::get_memory_glue();

        let sqls = vec![
            "DROP TABLE IF EXISTS notifications;",
            r#"
        CREATE TABLE notifications (
            id INTEGER, description TEXT, 
            created_at TIMESTAMP, 
            work_expired_at TIMESTAMP, break_expired_at TIMESTAMP,
        );"#,
        ];

        for sql in sqls {
            let output = test_glue.execute(sql).unwrap();
            debug!("output: {:?}", output);
        }
    });

    let mut id_manager: u16 = 1;
    let mut hash_map: HashMap<u16, JoinHandle<()>> = HashMap::new();

    let (tx, mut rx) = mpsc::channel::<Message>(64);

    let tx_for_command = tx.clone();
    tokio::spawn(async move {
        let mut t = 1;

        loop {
            debug!("inside spawn blocking");
            let command = read_command(t);
            t += 1;
            debug!("user input: {}", &command);

            let (oneshot_tx, oneshot_rx) = oneshot::channel::<String>();

            debug!("command: {:?}", command);

            let _ = tx_for_command
                .send(Message::UserInput {
                    command,
                    oneshot_tx,
                })
                .await;

            let result = oneshot_rx.await.unwrap();

            if !result.is_empty() {
                info!("{}", result);
            }
        }
    });

    while let Some(message) = rx.recv().await {
        match message {
            Message::UserInput {
                command,
                oneshot_tx,
            } => {
                let app = argument::get_app();
                let input = command.split_whitespace();

                debug!("input: {:?}", input);

                let matches = match app.get_matches_from_safe(input) {
                    Err(err) => {
                        match err.kind {
                            // HelpDisplayed has help message in error
                            clap::ErrorKind::HelpDisplayed => {
                                print!("\n{}\n", err);
                                let _ = oneshot_tx.send("".to_string());
                            }
                            // clap automatically print version string with out newline.
                            clap::ErrorKind::VersionDisplayed => {
                                println!();
                                let _ = oneshot_tx.send("".to_string());
                            }
                            _ => {
                                print!("\n{}\n", err);
                                let _ = oneshot_tx.send("".to_string());
                            }
                        }
                        continue;
                    }
                    Ok(arg) => arg,
                };

                match matches.subcommand() {
                    (CREATE, Some(sub_matches)) => {
                        let (work_time, break_time) = if sub_matches.is_present("default") {
                            (DEFAULT_WORK_TIME, DEFAULT_BREAK_TIME)
                        } else {
                            let work_time = parse_arg::<u16>(sub_matches, "work")?;
                            let break_time = parse_arg::<u16>(sub_matches, "break")?;

                            (work_time, break_time)
                        };

                        debug!("work_time: {}", work_time);
                        debug!("break_time: {}", break_time);

                        if work_time == 0 && break_time == 0 {
                            let _ = oneshot_tx.send(
                                String::from("work_time and break_time both can not be zero both")
                                    .to_string(),
                            );
                            continue;
                        }

                        let id = get_new_id(&mut id_manager);

                        let tx = tx.clone();
                        let _ = tx
                            .send(Message::Create {
                                id,
                                work_time,
                                break_time,
                                oneshot_tx,
                            })
                            .await;
                    }
                    (DELETE, Some(sub_matches)) => {
                        if sub_matches.is_present("id") {
                            // delete one
                            let id = parse_arg::<u16>(sub_matches, "id")?;

                            let tx = tx.clone();
                            let _ = tx.send(Message::Delete { id, oneshot_tx }).await;
                        } else {
                            // delete all
                            let tx = tx.clone();
                            let _ = tx.send(Message::DeleteAll { oneshot_tx }).await;
                        }
                    }
                    (LIST, Some(_)) => {
                        let tx = tx.clone();
                        let _ = tx.send(Message::Query { oneshot_tx }).await;
                    }
                    (LS, Some(_)) => {
                        let tx = tx.clone();
                        let _ = tx.send(Message::Query { oneshot_tx }).await;
                    }
                    (TEST, Some(_)) => {
                        let tx = tx.clone();
                        let _ = tx.send(Message::NotificationTest { oneshot_tx }).await;
                    }
                    (CLEAR, Some(_)) => {
                        print!("\x1B[2J");
                        let _ = oneshot_tx.send("".to_string());
                    }
                    (EXIT, Some(_)) => {
                        process::exit(0);
                    }
                    _ => {}
                }
            }
            Message::Create {
                id,
                work_time,
                break_time,
                oneshot_tx,
            } => {
                db::create_notification(&mut glue, &Notification::new(id, work_time, break_time))
                    .await;

                let tx = tx.clone();
                let handle =
                    spawn_notification(tx, id, work_time, break_time, configuration.clone());
                hash_map.insert(id, handle);

                let _ = oneshot_tx.send(format!("Notification (id: {}) created", id).to_string());
            }
            Message::Delete { id, oneshot_tx } => {
                debug!("Message::Delete called! {}", id);

                delete_notification(id, &mut hash_map, &mut glue).await?;

                debug!("Message::Delete done");
                let _ = oneshot_tx.send(format!("Notification (id: {}) deleted", id).to_string());
            }
            Message::SilentDelete { id } => {
                delete_notification(id, &mut hash_map, &mut glue).await?;
            }
            Message::DeleteAll { oneshot_tx } => {
                debug!("Message:DeleteAll called!");

                for (_, handle) in hash_map.iter() {
                    handle.abort();
                }
                db::delete_all_notification(&mut glue).await;

                debug!("Message::DeleteAll done");
                let _ = oneshot_tx.send(String::from("All notifications deleted"));
            }
            Message::Query { oneshot_tx } => {
                debug!("Message::Query called!");

                db::list_notification(&mut glue).await;

                debug!("Message::Query done");
                let _ = oneshot_tx.send(String::from("Query succeed"));
            }
            Message::NotificationTest { oneshot_tx } => {
                debug!("Message:NotificationTest called!");

                notify_work(&configuration.clone()).await?;

                debug!("Message:NotificationTest done");
                let _ = oneshot_tx.send(String::from("NotificationTest called"));
            }
        }
    }

    Ok(())
}

fn get_package_name() -> String {
    let package_name = env!("CARGO_PKG_NAME");
    package_name.replace("-", "_")
}

fn initialize_logging() {
    let package_name = &get_package_name();

    if cfg!(debug_assertions) {
        env_logger::Builder::from_default_env()
            .filter(Some(package_name), log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter(Some(package_name), log::LevelFilter::Info)
            .init();
    }
}

fn get_new_id(id_manager: &mut u16) -> u16 {
    let id = *id_manager;
    *id_manager += 1;

    id
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

async fn delete_notification(
    id: u16,
    hash_map: &mut HashMap<u16, JoinHandle<()>>,
    glue: &mut Glue<Key, MemoryStorage>,
) -> Result<(), Box<dyn Error>> {
    hash_map
        .get(&id)
        .ok_or(format!("failed to corresponding task (id: {})", &id))?
        .abort();

    hash_map
        .remove(&id)
        .ok_or(format!("failed to revmoe id ({})", id))?;

    db::delete_notification(glue, id).await;

    Ok(())
}

fn spawn_notification(
    tx: Sender<Message>,
    id: u16,
    work_time: u16,
    break_time: u16,
    configuration: Arc<Configuration>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        debug!("id: {}, task started", id);

        if work_time > 0 {
            let wt = tokio::time::Duration::from_secs(work_time as u64);
            sleep(wt).await;
            debug!("id ({}), work time ({}) done", id, work_time);

            let _ = notify_work(&configuration).await;
        }

        if break_time > 0 {
            let bt = tokio::time::Duration::from_secs(break_time as u64);
            sleep(bt).await;
            debug!("id ({}), break time ({}) done", id, break_time);

            let _ = notify_break(&configuration).await;
        }

        let _ = tx.send(Message::SilentDelete { id }).await;

        debug!("id: {}, notification work time done!", id);
    })
}
