use chrono::Utc;
use clap::{ArgMatches, Command, ErrorKind};
use std::error::Error;
use std::process;
use std::str::SplitWhitespace;
use std::sync::Arc;
use tabled::{Style, TableIteratorExt};

use crate::command::handler::HandleResult;
use crate::command::util;
use crate::command::{self, action::ActionType};
use crate::notification::notify::notify_work;
use crate::notification::{delete_notification, get_new_notification};
use crate::{configuration::Configuration, ArcGlue};
use crate::{db, spawn_notification, ArcTaskMap};

pub async fn handle(
    user_input: &str,
    id_manager: &mut u16,
    notification_task_map: &ArcTaskMap,
    glue: &ArcGlue,
    configuration: &Arc<Configuration>,
) -> Result<Vec<String>, Box<dyn Error>> {
    let command = command::get_main_command();
    let input = user_input.split_whitespace();
    let mut output_accumulator = Vec::new();
    debug!("input: {:?}", input);

    let matches = match get_matches(command, input, &mut output_accumulator)? {
        Some(args) => args,
        None => return Ok(output_accumulator),
    };

    let (action_type, sub_matches) = matches
        .subcommand()
        .ok_or(Box::from("subcommand wasn't present at runtime") as Box<dyn Error>)
        .and_then(|(s, sub_matches)| ActionType::parse(s).map(|s| (s, sub_matches)))?;

    match action_type {
        ActionType::Create => {
            handle_create(
                sub_matches,
                configuration,
                notification_task_map,
                glue,
                id_manager,
                &mut output_accumulator,
            )
            .await?
        }
        ActionType::Queue => {
            handle_queue(
                sub_matches,
                configuration,
                notification_task_map,
                glue,
                id_manager,
                &mut output_accumulator,
            )
            .await?
        }
        ActionType::Delete => {
            handle_delete(
                sub_matches,
                notification_task_map,
                glue,
                &mut output_accumulator,
            )
            .await?
        }
        ActionType::List => handle_list(glue, &mut output_accumulator).await?,
        ActionType::Test => handle_test(configuration, &mut output_accumulator).await?,
        ActionType::History => handle_history(glue, &mut output_accumulator).await?,
        ActionType::Exit => process::exit(0),
        ActionType::Clear => print!("\x1B[2J\x1B[1;1H"),
    }

    Ok(output_accumulator)
}

async fn handle_create(
    matches: &ArgMatches,
    configuration: &Arc<Configuration>,
    notification_task_map: &ArcTaskMap,
    glue: &ArcGlue,
    id_manager: &mut u16,
    output_accumulator: &mut Vec<String>,
) -> HandleResult {
    let notification = get_new_notification(matches, id_manager, Utc::now())?;

    match notification {
        Some(notification) => {
            let id = notification.get_id();
            db::create_notification(glue.clone(), &notification).await;

            let handle = spawn_notification(
                configuration.clone(),
                notification_task_map.clone(),
                glue.clone(),
                notification,
            );

            notification_task_map.lock().unwrap().insert(id, handle);
            println!("Notification (id: {}) created", id);
            output_accumulator.push(format!("Notification (id: {}) created", id));

            Ok(())
        }
        None => {
            output_accumulator.push(String::from(
                "work_time and break_time both can not be zero both",
            ));

            Ok(())
        }
    }
}

async fn handle_queue(
    matches: &ArgMatches,
    configuration: &Arc<Configuration>,
    notification_task_map: &ArcTaskMap,
    glue: &ArcGlue,
    id_manager: &mut u16,
    output_accumulator: &mut Vec<String>,
) -> HandleResult {
    let created_at = match db::read_last_expired_notification(glue.clone()).await {
        Some(n) => {
            debug!("last_expired_notification: {:?}", &n);

            let (_, _, _, _, _, work_expired_at, break_expired_at) = n.get_values();
            work_expired_at.max(break_expired_at)
        }
        None => Utc::now(),
    };

    let notification = get_new_notification(matches, id_manager, created_at)?;
    match notification {
        Some(notification) => {
            let id = notification.get_id();
            db::create_notification(glue.clone(), &notification).await;

            notification_task_map.lock().unwrap().insert(
                id,
                spawn_notification(
                    configuration.clone(),
                    notification_task_map.clone(),
                    glue.clone(),
                    notification,
                ),
            );
            println!("Notification (id: {}) created and queued", id);
            output_accumulator.push(format!("Notification (id: {}) created and queued", id));

            Ok(())
        }
        None => Ok(()),
    }
}

async fn handle_delete(
    sub_matches: &ArgMatches,
    notification_task_map: &ArcTaskMap,
    glue: &ArcGlue,
    output_accumulator: &mut Vec<String>,
) -> HandleResult {
    if sub_matches.is_present("id") {
        // delete one
        let id = util::parse_arg::<u16>(sub_matches, "id")?;
        debug!("Message::Delete called! {}", id);

        match delete_notification(id, notification_task_map.clone(), glue.clone()).await {
            Ok(_) => {
                println!("Notification (id: {}) deleted", id);
                output_accumulator.push(format!("Notification (id: {}) deleted", id));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                output_accumulator.push(format!("Error: {}", e));
            }
        };
        debug!("Message::Delete done");
    } else {
        // delete all
        debug!("Message:DeleteAll called!");

        for (_, handle) in notification_task_map.lock().unwrap().iter() {
            handle.abort();
        }
        db::delete_and_archive_all_notification(glue.clone()).await;
        println!("All Notifications deleted");
        output_accumulator.push(String::from("All Notifications deleted"));
        debug!("Message::DeleteAll done");
    }

    Ok(())
}

async fn handle_test(
    configuration: &Arc<Configuration>,
    output_accumulator: &mut Vec<String>,
) -> HandleResult {
    debug!("Message:NotificationTest called!");
    let report = notify_work(&configuration.clone()).await?;
    info!("\n{}", report);
    output_accumulator.push(format!("\n{}", report));

    debug!("Message:NotificationTest done");
    println!("Notification Test called");
    output_accumulator.push(String::from("Notification Test called"));

    Ok(())
}

async fn handle_list(glue: &ArcGlue, output_accumulator: &mut Vec<String>) -> HandleResult {
    debug!("Message::List called!");
    let notifications = db::list_notification(glue.clone()).await;
    debug!("Message::List done");

    let table = notifications
        .table()
        .with(Style::modern().horizontal_off())
        .to_string();
    info!("\n{}", table);
    output_accumulator.push(format!("\n{}", table));

    println!("List succeed");
    output_accumulator.push(String::from("List succeed"));

    Ok(())
}

async fn handle_history(glue: &ArcGlue, output_accumulator: &mut Vec<String>) -> HandleResult {
    debug!("Message:History called!");
    let archived_notifications = db::list_archived_notification(glue.clone()).await;
    debug!("Message:History done!");

    let table = archived_notifications
        .table()
        .with(Style::modern().horizontal_off())
        .to_string();
    info!("\n{}", table);
    output_accumulator.push(format!("\n{}", table));

    println!("History succeed");
    output_accumulator.push(String::from("History succeed"));

    Ok(())
}

fn get_matches(
    command: Command,
    input: SplitWhitespace,
    output_accumulator: &mut Vec<String>,
) -> Result<Option<ArgMatches>, Box<dyn Error>> {
    match command.try_get_matches_from(input) {
        Ok(args) => Ok(Some(args)),
        Err(err) => {
            match err.kind() {
                // DisplayHelp has help message in error
                ErrorKind::DisplayHelp => {
                    // print!("\n{}\n", err);
                    // TODO(young): test format! works well
                    output_accumulator.push(format!("\n{}\n", err));
                    return Ok(None);
                }
                // clap automatically print version string with out newline.
                ErrorKind::DisplayVersion => {
                    // println!();
                    output_accumulator.push(String::from(""));
                    return Ok(None);
                }
                _ => {
                    print!("\n error while handling the input, {}\n", err);
                }
            }

            Err(Box::new(err))
        }
    }
}
