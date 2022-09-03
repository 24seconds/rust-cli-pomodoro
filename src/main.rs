use chrono::Utc;
use gluesql::prelude::{Glue, MemoryStorage};
use std::collections::HashMap;
use std::error::Error;
use std::io::{self};
use std::sync::{Arc, Mutex};
use tokio::time::sleep;
use tokio::{net::UnixDatagram, sync::mpsc};
use tokio::{sync::mpsc::Sender, task::JoinHandle};

mod command;
mod database;
mod notification;
use database as db;
mod configuration;
mod error;
mod ipc;
mod logging;
mod report;

use crate::error::ConfigurationError;
use crate::ipc::{create_client_uds, create_server_uds, Bincodec, MessageRequest, MessageResponse};
use crate::notification::archived_notification;
use crate::notification::notify::{notify_break, notify_work};
use crate::notification::Notification;
use crate::{
    command::{handler, util, CommandType},
    ipc::{get_uds_address, UdsType},
};
use crate::{
    configuration::{get_configuration, Configuration},
    ipc::UdsMessage,
};

#[macro_use]
extern crate log;

// key: notification id, value: spawned notification task
pub type TaskMap = HashMap<u16, JoinHandle<()>>;
pub type ArcGlue = Arc<Mutex<Glue<MemoryStorage>>>;
pub type ArcTaskMap = Arc<Mutex<TaskMap>>;

#[derive(Debug)]
struct UserInput {
    pub input: String,
    // pub oneshot_tx: oneshot::Sender<String>,
    pub source: InputSource,
}

#[derive(Debug)]
enum InputSource {
    StandardInput,
    UnixDomainSocket,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    logging::initialize_logging();
    debug!("debug test, start pomodoro...");

    let command_type = detect_command_type().await?;

    match command_type {
        CommandType::StartUp(config) => {
            info!("start pomodoro...");
            debug!("CommandType::StartUp");

            let glue = initialize_db().await;
            let mut id_manager: u16 = 1;
            let hash_map: Arc<Mutex<TaskMap>> = Arc::new(Mutex::new(HashMap::new()));
            let (user_input_tx, mut user_input_rx) = mpsc::channel::<UserInput>(64);

            let stdin_tx = user_input_tx.clone();
            // TODO(young): handle tokio::spawn return value nicely so that we can use `?` inside
            let stdinput_handle = spawn_stdinput_handler(stdin_tx);

            // handle uds
            let uds_input_tx = user_input_tx.clone();

            let server_uds_option = create_server_uds().await.unwrap();
            let server_tx = match server_uds_option {
                Some(uds) => {
                    let server_uds = Arc::new(uds);
                    let (server_rx, server_tx) = (server_uds.clone(), server_uds.clone());
                    let uds_input_handle =
                        spawn_uds_input_handler(uds_input_tx, server_tx, server_rx);

                    Some(server_uds)
                }
                None => None,
            };

            // TODO(young) handle `rx.recv().await` returns None case
            // TODO(young): handle tokio::spawn return value nicely so that we can use `?` inside
            while let Some(user_input) = user_input_rx.recv().await {
                // extract input
                let input = user_input.input.as_str();
                debug!("input: {:?}", input);

                // handle input
                match handler::user_input::handle(input, &mut id_manager, &hash_map, &glue, &config)
                    .await
                {
                    Ok(mut output) => match user_input.source {
                        InputSource::StandardInput => {}
                        InputSource::UnixDomainSocket => {
                            if let Some(ref server_tx) = server_tx {
                                let client_addr = get_uds_address(UdsType::Client);
                                ipc::send_to(
                                    server_tx,
                                    client_addr,
                                    MessageResponse::new(output.take_body())
                                        .encode()?
                                        .as_slice(),
                                )
                                .await;
                            }
                        }
                    },
                    Err(e) => {
                        println!("There was an error analyzing the input: {}", e);

                        match user_input.source {
                            InputSource::StandardInput => {}
                            InputSource::UnixDomainSocket => {
                                if let Some(ref server_tx) = server_tx {
                                    let client_addr = get_uds_address(UdsType::Client);
                                    ipc::send_to(
                                        server_tx,
                                        client_addr,
                                        MessageResponse::new(vec![format!(
                                            "There was an error analyzing the input: {}",
                                            e
                                        )])
                                        .encode()?
                                        .as_slice(),
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                }

                debug!("input: {:?}", user_input);
                util::print_start_up();
            }
        }
        CommandType::UdsClient(matches) => {
            debug!("CommandType::UdsClient");
            let socket = create_client_uds().await?;

            handler::uds_client::handle(matches, socket).await?;
        }
    }

    debug!("handle_uds_client_command called successfully");

    Ok(())
}

async fn detect_command_type() -> Result<CommandType, ConfigurationError> {
    let matches = command::get_start_and_uds_client_command().get_matches();
    debug!("handle_uds_client_command, matches: {:?}", &matches);

    let command_type = match (&matches).subcommand().is_none() {
        true => CommandType::StartUp(get_configuration(&matches)?),
        false => CommandType::UdsClient(matches),
    };

    Ok(command_type)
}

async fn initialize_db() -> Arc<Mutex<Glue<MemoryStorage>>> {
    let glue = Arc::new(Mutex::new(db::get_memory_glue()));
    db::initialize(glue.clone()).await;

    glue
}

// TODO(young): refactor and move to proper place
pub fn spawn_notification(
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
                util::write_output(&mut io::stdout());
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
                util::write_output(&mut io::stdout());
            }
        }

        let result = notification::delete_notification(id, hash_map, glue.clone()).await;
        if result.is_err() {
            trace!("error occurred while deleting notification");
        }

        debug!("id: {}, notification work time done!", id);
    })
}

fn spawn_stdinput_handler(tx: Sender<UserInput>) -> JoinHandle<()> {
    tokio::spawn(async move {
        util::print_start_up();

        loop {
            debug!("inside stdin task");
            let user_input = util::read_input(&mut io::stdin().lock());
            debug!("user input: {}", &user_input);

            let _ = tx
                .send(UserInput {
                    input: user_input,
                    source: InputSource::StandardInput,
                })
                .await;
        }
    })
}

fn spawn_uds_input_handler(
    uds_tx: Sender<UserInput>,
    server_tx: Arc<UnixDatagram>,
    server_rx: Arc<UnixDatagram>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        // TODO(young) handle result
        let rx = server_rx;
        let mut buf = vec![0u8; 256];
        debug!("rx is initialized successfully");
        loop {
            debug!("inside unix domain socket task");
            // TODO(young) handle result
            let (size, addr) = rx.recv_from(&mut buf).await.unwrap();
            debug!("size: {:?}, addr: {:?}", size, addr);

            if let Some(path) = addr.as_pathname() {
                // ignore request from unnamed address
                if path != get_uds_address(ipc::UdsType::Client).as_path() {
                    debug!("addr is different");
                    continue;
                }
            }

            let uds_message = UdsMessage::decode(&buf[..size]).unwrap();
            match uds_message {
                UdsMessage::Public(message) => {
                    let user_input: UserInput = MessageRequest::into(message);
                    debug!("user_input: {:?}", user_input);

                    let _ = uds_tx.send(user_input).await.unwrap();
                }
                UdsMessage::Internal(message) => {
                    debug!("internal_message ok, {:?}", message);
                    match message {
                        ipc::internal::Message::Ping => {
                            ipc::send_to(
                                &server_tx,
                                addr.as_pathname().unwrap().to_path_buf(),
                                ipc::internal::Message::Pong.encode().unwrap().as_slice(),
                            )
                            .await;
                        }
                        ipc::internal::Message::Pong => {}
                    }
                }
            }
        }
    })
}
