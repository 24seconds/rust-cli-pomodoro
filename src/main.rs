use chrono::Utc;
use clap_v3::ArgMatches;
use gluesql::{
    memory_storage::Key,
    prelude::{Glue, MemoryStorage},
};
use std::collections::HashMap;
use std::error::Error;
use std::io::{self};
use std::process;
use std::sync::{Arc, Mutex};
use tabled::{Style, TableIteratorExt};
use tokio::task::JoinHandle;
use tokio::time::sleep;

mod archived_notification;
mod argument;
mod database;
mod notification;
use database as db;
mod configuration;
mod error;
mod input_handler;
mod logging;
mod notify;

use crate::argument::{parse_arg, CLEAR, CREATE, DELETE, EXIT, HISTORY, LIST, LS, Q, QUEUE, TEST};
use crate::configuration::{
    generate_configuration_report, initialize_configuration, Configuration,
};
use crate::notification::Notification;
use crate::notify::{notify_break, notify_work};

#[macro_use]
extern crate log;

type TaskMap = HashMap<u16, JoinHandle<()>>;
pub type ArcGlue = Arc<Mutex<Glue<Key, MemoryStorage>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    logging::initialize_logging();

    info!("info test, start pomodoro...");
    debug!("debug test, start pomodoro...");

    let config_matches = argument::get_config_command().get_matches();
    let credential_file_path = config_matches.value_of("config");

    let (configuration, config_error) = initialize_configuration(credential_file_path)?;
    let report = generate_configuration_report(&configuration, config_error);
    info!("\nconfig flag result!\n{}", report);
    let configuration = Arc::new(configuration);

    let glue = Arc::new(Mutex::new(db::get_memory_glue()));
    db::initialize(glue.clone()).await;

    let mut id_manager: u16 = 1;
    let hash_map: Arc<Mutex<TaskMap>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        debug!("inside spawn blocking");
        let user_input = input_handler::read_input(&mut io::stdout(), &mut io::stdin().lock());

        debug!("user input: {}", &user_input);
        if let Err(e) = analyze_input(
            &user_input,
            &mut id_manager,
            &hash_map,
            &glue,
            &configuration,
        )
        .await
        {
            println!("There was an error analyzing the input: {}", e);
        };
    }
}

async fn analyze_input(
    user_input: &str,
    id_manager: &mut u16,
    hash_map: &Arc<Mutex<TaskMap>>,
    glue: &Arc<Mutex<Glue<Key, MemoryStorage>>>,
    configuration: &Arc<Configuration>,
) -> Result<(), Box<dyn Error>> {
    let command = argument::get_command();
    let input = user_input.split_whitespace();
    debug!("input: {:?}", input);

    let matches = match command.try_get_matches_from(input) {
        Ok(args) => args,
        Err(err) => {
            match err.kind() {
                // DisplayHelp has help message in error
                clap_v3::ErrorKind::DisplayHelp => {
                    print!("\n{}\n", err);
                    return Ok(());
                }
                // clap automatically print version string with out newline.
                clap_v3::ErrorKind::DisplayVersion => {
                    println!();
                    return Ok(());
                }
                _ => {
                    print!("\n error while handling the input, {}\n", err);
                }
            }
            return Err(Box::new(err));
        }
    };

    match matches.subcommand() {
        Some((CREATE, sub_matches)) => {
            handle_create(sub_matches, configuration, hash_map, glue, id_manager).await?;
        }
        Some((QUEUE, sub_matches)) | Some((Q, sub_matches)) => {
            handle_queue(sub_matches, configuration, hash_map, glue, id_manager).await?;
        }
        Some((DELETE, sub_matches)) => {
            if sub_matches.is_present("id") {
                // delete one
                let id = parse_arg::<u16>(sub_matches, "id")?;

                debug!("Message::Delete called! {}", id);

                let result = delete_notification(id, hash_map.clone(), glue.clone()).await;

                match result {
                    Ok(_) => println!("Notification (id: {}) deleted", id),
                    Err(e) => eprintln!("Error: {}", e),
                };
                debug!("Message::Delete done");
            } else {
                // delete all
                debug!("Message:DeleteAll called!");

                let hash_map = hash_map.lock().unwrap();
                for (_, handle) in hash_map.iter() {
                    handle.abort();
                }
                db::delete_and_archive_all_notification(glue.clone()).await;
                println!("All Notifications deleted");
                debug!("Message::DeleteAll done");
            }
        }
        Some((LS, _)) | Some((LIST, _)) => {
            handle_list(glue).await;
        }
        Some((HISTORY, _)) => {
            handle_history(glue).await;
        }
        Some((TEST, _)) => {
            handle_test(configuration).await?;
        }
        Some((CLEAR, _)) => {
            print!("\x1B[2J\x1B[1;1H");
        }
        Some((EXIT, _)) => {
            process::exit(0);
        }
        _ => (),
    };
    Ok(())
}

async fn handle_list(glue: &Arc<Mutex<Glue<Key, MemoryStorage>>>) {
    debug!("Message::List called!");
    let notifications = db::list_notification(glue.clone()).await;
    debug!("Message::List done");

    let table = notifications
        .table()
        .with(Style::modern().horizontal_off())
        .to_string();
    info!("\n{}", table);

    println!("List succeed");
}

async fn handle_history(glue: &Arc<Mutex<Glue<Key, MemoryStorage>>>) {
    debug!("Message:History called!");
    let archived_notifications = db::list_archived_notification(glue.clone()).await;
    debug!("Message:History done!");

    let table = archived_notifications
        .table()
        .with(Style::modern().horizontal_off())
        .to_string();
    info!("\n{}", table);

    println!("History succeed");
}

async fn handle_test(configuration: &Arc<Configuration>) -> Result<(), Box<dyn Error>> {
    debug!("Message:NotificationTest called!");
    let report = notify_work(&configuration.clone()).await?;
    info!("\n{}", report);
    debug!("Message:NotificationTest done");
    println!("Notification Test called");

    Ok(())
}

async fn handle_create(
    matches: &ArgMatches,
    configuration: &Arc<Configuration>,
    hash_map: &Arc<Mutex<TaskMap>>,
    glue: &ArcGlue,
    id_manager: &mut u16,
) -> Result<(), Box<dyn Error>> {
    let notification = input_handler::get_new_notification(matches, id_manager, Utc::now())?;
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
    println!("Notification (id: {}) created", id);
    Ok(())
}

async fn handle_queue(
    matches: &ArgMatches,
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

    let notification = input_handler::get_new_notification(matches, id_manager, created_at)?;
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
    println!("Notification (id: {}) created and queued", id);
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

    db::delete_and_archive_notification(glue, id).await;

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

            // TODO(young): handle notify report err
            let result = notify_work(&configuration).await;
            if let Ok(report) = result {
                info!("\n{}", report);
                println!("Notification report generated");
                input_handler::write_output(&mut io::stdout());
            }
        }

        if break_time > 0 {
            let bt = tokio::time::Duration::from_secs(break_time as u64 * 60);
            sleep(bt).await;
            debug!("id ({}), break time ({}) done", id, break_time);

            // TODO(young): handle notify report err
            let result = notify_break(&configuration).await;
            if let Ok(report) = result {
                info!("\n{}", report);
                println!("Notification report generated");
                input_handler::write_output(&mut io::stdout());
            }
        }

        let result = delete_notification(id, hash_map, glue.clone()).await;
        if result.is_err() {
            trace!("error occurred while deleting notification");
        }

        debug!("id: {}, notification work time done!", id);
    })
}
