use gluesql::{memory_storage::Key, Glue, MemoryStorage};
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::process;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::sleep;

mod argument;
mod database;
mod notification;
use database as db;
mod configuration;

use crate::argument::{
    parse_arg, CLEAR, CREATE, DEFAULT_BREAK_TIME, DEFAULT_WORK_TIME, DELETE, EXIT, LIST, LS, TEST,
};
use crate::configuration::{initialize_configuration, Configuration};
use crate::notification::{notify_break, notify_work, Notification};

#[macro_use]
extern crate log;

type TaskMap = HashMap<u16, JoinHandle<()>>;
type ArcGlue = Arc<Mutex<Glue<Key, MemoryStorage>>>;

struct UserInput {
    pub command: String,
    pub oneshot_tx: oneshot::Sender<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    initialize_logging();

    let config_matches = argument::get_config_app().get_matches();
    let credential_file_path = config_matches.value_of("config");

    let configuration = initialize_configuration(credential_file_path)?;
    let configuration = Arc::new(configuration);

    info!("info test, start pomodoro...");
    debug!("debug test, start pomodoro...");

    let glue = Arc::new(Mutex::new(db::get_memory_glue()));
    db::initialize(glue.clone()).await;

    let mut id_manager: u16 = 1;
    let hash_map: Arc<Mutex<TaskMap>> = Arc::new(Mutex::new(HashMap::new()));

    let (tx, mut rx) = mpsc::channel::<UserInput>(64);
    let tx_for_command = tx.clone();

    tokio::spawn(async move {
        loop {
            debug!("inside spawn blocking");
            let command = read_command();
            debug!("user input: {}", &command);

            let (oneshot_tx, oneshot_rx) = oneshot::channel::<String>();

            debug!("command: {:?}", command);

            let _ = tx_for_command
                .send(UserInput {
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

    while let Some(input) = rx.recv().await {
        let UserInput {
            command,
            oneshot_tx,
        } = input;

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

                db::create_notification(
                    glue.clone(),
                    &Notification::new(id, work_time, break_time),
                )
                .await;

                let handle = spawn_notification(
                    id,
                    work_time,
                    break_time,
                    configuration.clone(),
                    hash_map.clone(),
                    glue.clone(),
                );

                let mut hash_map = hash_map.lock().unwrap();
                hash_map.insert(id, handle);

                let _ = oneshot_tx.send(format!("Notification (id: {}) created", id).to_string());
            }
            (DELETE, Some(sub_matches)) => {
                if sub_matches.is_present("id") {
                    // delete one
                    let id = parse_arg::<u16>(sub_matches, "id")?;

                    debug!("Message::Delete called! {}", id);

                    let result = delete_notification(id, hash_map.clone(), glue.clone()).await;

                    let message = match result {
                        Ok(_) => format!("Notification (id: {}) deleted", id).to_string(),
                        Err(e) => format!("Error: {}", e).to_string(),
                    };

                    debug!("Message::Delete done");
                    let _ = oneshot_tx.send(message);
                } else {
                    // delete all
                    debug!("Message:DeleteAll called!");

                    let hash_map = hash_map.lock().unwrap();
                    for (_, handle) in hash_map.iter() {
                        handle.abort();
                    }
                    db::delete_all_notification(glue.clone()).await;

                    debug!("Message::DeleteAll done");
                    let _ = oneshot_tx.send(String::from("All notifications deleted"));
                }
            }
            (LS, Some(_)) | (LIST, Some(_)) => {
                debug!("Message::Query called!");
                db::list_notification(glue.clone()).await;
                debug!("Message::Query done");

                let _ = oneshot_tx.send(String::from("Query succeed"));
            }
            (TEST, Some(_)) => {
                debug!("Message:NotificationTest called!");
                notify_work(&configuration.clone()).await?;
                debug!("Message:NotificationTest done");

                let _ = oneshot_tx.send(String::from("NotificationTest called"));
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

async fn delete_notification(
    id: u16,
    hash_map: Arc<Mutex<TaskMap>>,
    glue: ArcGlue,
) -> Result<(), Box<dyn Error>> {
    let notification = db::read_notification(glue.clone(), id).await;
    if notification.is_none() {
        return Err(format!(
            "deleting id ({}) failed. Corresponding notification does not exist",
            id
        )
        .into());
    }

    {
        let mut hash_map = hash_map.lock().unwrap();

        hash_map
            .get(&id)
            .ok_or(format!("failed to corresponding task (id: {})", &id))?
            .abort();

        hash_map
            .remove(&id)
            .ok_or(format!("failed to remove id ({})", id))?;
    }

    db::delete_notification(glue, id).await;

    Ok(())
}

fn spawn_notification(
    id: u16,
    work_time: u16,
    break_time: u16,
    configuration: Arc<Configuration>,
    hash_map: Arc<Mutex<TaskMap>>,
    glue: ArcGlue,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        debug!("id: {}, task started", id);

        if work_time > 0 {
            let wt = tokio::time::Duration::from_secs(work_time as u64 * 60);
            sleep(wt).await;
            debug!("id ({}), work time ({}) done", id, work_time);

            let _ = notify_work(&configuration).await;
        }

        if break_time > 0 {
            let bt = tokio::time::Duration::from_secs(break_time as u64 * 60);
            sleep(bt).await;
            debug!("id ({}), break time ({}) done", id, break_time);

            let _ = notify_break(&configuration).await;
        }

        let _ = delete_notification(id, hash_map, glue.clone()).await;

        debug!("id: {}, notification work time done!", id);
    })
}
