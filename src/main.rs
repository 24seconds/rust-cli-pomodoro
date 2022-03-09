use chrono::{DateTime, Utc};
use clap::ArgMatches;
use gluesql::{
    prelude::{Glue, MemoryStorage},
    storages::memory_storage::Key,
};
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::process;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::sleep;

mod argument;
mod database;
mod notification;
use database as db;
mod configuration;

use crate::argument::{
    parse_arg, CLEAR, CREATE, DEFAULT_BREAK_TIME, DEFAULT_WORK_TIME, DELETE, EXIT, LIST, LS, Q,
    QUEUE, TEST,
};
use crate::configuration::{initialize_configuration, Configuration};
use crate::notification::{notify_break, notify_work, Notification};

#[macro_use]
extern crate log;

type TaskMap = HashMap<u16, JoinHandle<()>>;
type ArcGlue = Arc<Mutex<Glue<Key, MemoryStorage>>>;

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

    loop {
        debug!("inside spawn blocking");
        let command = read_command();
        debug!("user input: {}", &command);
        if let Err(e) =
            analyze_input(&command, &mut id_manager, &hash_map, &glue, &configuration).await
        {
            println!("There was an error analyzing the input: {}", e);
        };
    }
}

async fn analyze_input(
    command: &str,
    id_manager: &mut u16,
    hash_map: &Arc<Mutex<TaskMap>>,
    glue: &Arc<Mutex<Glue<Key, MemoryStorage>>>,
    configuration: &Arc<Configuration>,
) -> Result<(), Box<dyn Error>> {
    let app = argument::get_app();
    let input = command.split_whitespace();
    debug!("input: {:?}", input);
    let matches = match app.get_matches_from_safe(input) {
        Ok(args) => args,
        Err(err) => {
            match err.kind {
                // HelpDisplayed has help message in error
                clap::ErrorKind::HelpDisplayed => {
                    print!("\n{}\n", err);
                }
                // clap automatically print version string with out newline.
                clap::ErrorKind::VersionDisplayed => {
                    println!();
                }
                _ => {
                    print!("\n{}\n", err);
                }
            }
            return Err(Box::new(err));
        }
    };

    match matches.subcommand() {
        (CREATE, Some(sub_matches)) => {
            create_notification(sub_matches, configuration, hash_map, glue, id_manager).await?;
        }
        (QUEUE, Some(sub_matches)) | (Q, Some(sub_matches)) => {
            queue_notification(sub_matches, configuration, hash_map, glue, id_manager).await?;
        }
        (DELETE, Some(sub_matches)) => {
            if sub_matches.is_present("id") {
                // delete one
                let id = parse_arg::<u16>(sub_matches, "id")?;

                debug!("Message::Delete called! {}", id);

                let result = delete_notification(id, hash_map.clone(), glue.clone()).await;

                match result {
                    Ok(_) => println!("{}", format!("Notification (id: {}) deleted", id)),
                    Err(e) => eprintln!("{}", format!("Error: {}", e)),
                };
                debug!("Message::Delete done");
            } else {
                // delete all
                debug!("Message:DeleteAll called!");

                let hash_map = hash_map.lock().unwrap();
                for (_, handle) in hash_map.iter() {
                    handle.abort();
                }
                db::delete_all_notification(glue.clone()).await;
                println!("All Notifications deleted");
                debug!("Message::DeleteAll done");
            }
        }
        (LS, Some(_)) | (LIST, Some(_)) => {
            debug!("Message::Query called!");
            db::list_notification(glue.clone()).await;
            debug!("Message::Query done");
            println!("Query succeed");
        }
        (TEST, Some(_)) => {
            debug!("Message:NotificationTest called!");
            notify_work(&configuration.clone()).await?;
            debug!("Message:NotificationTest done");
            println!("Notification Test called");
        }
        (CLEAR, Some(_)) => {
            print!("\x1B[2J\x1B[1;1H");
        }
        (EXIT, Some(_)) => {
            process::exit(0);
        }
        _ => (),
    };
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

fn get_new_notification(
    matches: &ArgMatches<'_>,
    id_manager: &mut u16,
    created_at: DateTime<Utc>,
) -> Result<Option<Notification>, Box<dyn Error>> {
    let (work_time, break_time) = if matches.is_present("default") {
        (DEFAULT_WORK_TIME, DEFAULT_BREAK_TIME)
    } else {
        let work_time = parse_arg::<u16>(matches, "work")?;
        let break_time = parse_arg::<u16>(matches, "break")?;

        (work_time, break_time)
    };

    debug!("work_time: {}", work_time);
    debug!("break_time: {}", break_time);

    if work_time == 0 && break_time == 0 {
        eprintln!("work_time and break_time both can not be zero both");
        // TODO: This shouldn't return Ok, since it is an error, but for now,
        // is just a "temporal fix" for returning from the function.
        return Ok(None);
    }

    let id = get_new_id(id_manager);

    Ok(Some(Notification::new(
        id, work_time, break_time, created_at,
    )))
}

async fn create_notification(
    matches: &ArgMatches<'_>,
    configuration: &Arc<Configuration>,
    hash_map: &Arc<Mutex<TaskMap>>,
    glue: &ArcGlue,
    id_manager: &mut u16,
) -> Result<(), Box<dyn Error>> {
    let notification = get_new_notification(matches, id_manager, Utc::now())?;
    let notification = match notification {
        Some(n) => n,
        None => return Ok(()),
    };
    let id = notification.get_id();

    db::create_notification(glue.clone(), &notification).await;

    let handle = spawn_notification(
        configuration.clone(),
        hash_map.clone(),
        glue.clone(),
        notification,
    );
    let mut hash_map = hash_map.lock().unwrap();
    hash_map.insert(id, handle);
    println!("{}", format!("Notification (id: {}) created", id));
    Ok(())
}

async fn queue_notification(
    matches: &ArgMatches<'_>,
    configuration: &Arc<Configuration>,
    hash_map: &Arc<Mutex<TaskMap>>,
    glue: &ArcGlue,
    id_manager: &mut u16,
) -> Result<(), Box<dyn Error>> {
    let last_expired_notification = db::read_last_expired_notification(glue.clone()).await;

    let created_at = match last_expired_notification {
        Some(n) => {
            debug!("last_expired_notification: {:?}", &n);

            let (_, _, _, _, _, work_expired_at, break_expired_at) = n.get_values();

            work_expired_at.max(break_expired_at)
        }
        None => Utc::now(),
    };

    let notification = get_new_notification(matches, id_manager, created_at)?;
    let notification = match notification {
        Some(n) => n,
        None => return Ok(()),
    };
    let id = notification.get_id();
    db::create_notification(glue.clone(), &notification).await;

    let handle = spawn_notification(
        configuration.clone(),
        hash_map.clone(),
        glue.clone(),
        notification,
    );
    let mut hash_map = hash_map.lock().unwrap();
    hash_map.insert(id, handle);
    println!(
        "{}",
        format!("Notification (id: {}) created and queued", id)
    );
    Ok(())
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
    configuration: Arc<Configuration>,
    hash_map: Arc<Mutex<TaskMap>>,
    glue: ArcGlue,
    notification: Notification,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let (id, _, work_time, break_time, _, _, _) = notification.get_values();
        debug!("id: {}, task started", id);

        let before_start_remaining = (notification.get_start_at() - Utc::now()).num_seconds();
        let before = tokio::time::Duration::from_secs(before_start_remaining as u64);
        debug!("before_start_remaining: {:?}", before_start_remaining);
        sleep(before).await;

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
