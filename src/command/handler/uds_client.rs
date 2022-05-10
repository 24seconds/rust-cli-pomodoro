use clap::ArgMatches;
use std::error::Error;
use tokio::net::UnixDatagram;

use crate::command::action::ActionType;
use crate::command::handler::HandleResult;
use crate::command::util;
use crate::ipc::{MessageRequest, MessageResponse};

const BUFFER_LENGTH: usize = 4096;

// TODO(young): handle error properly
pub async fn handle(matches: ArgMatches, socket: UnixDatagram) -> HandleResult {
    let (action_type, sub_matches) = matches
        .subcommand()
        .ok_or(Box::from("subcommand wasn't present at runtime") as Box<dyn Error>)
        .and_then(|(s, sub_matches)| ActionType::parse(s).map(|s| (s, sub_matches)))?;

    match action_type {
        ActionType::Create => handle_create(socket, sub_matches).await?,
        ActionType::Queue => handle_queue(socket, sub_matches).await?,
        ActionType::Delete => handle_delete(socket, sub_matches).await?,
        ActionType::List => handle_list(socket).await?,
        ActionType::Test => handle_test(socket).await?,
        ActionType::History => handle_history(socket).await?,
        ActionType::Exit | ActionType::Clear => {
            info!("Exit or Clear is not supported action for unix domain client")
        }
    }

    Ok(())
}

async fn handle_create(socket: UnixDatagram, sub_matches: &ArgMatches) -> HandleResult {
    let (work_time, break_time) = util::parse_work_and_break_time(sub_matches)?;

    socket
        .send(
            MessageRequest::Create {
                work: work_time,
                r#break: break_time,
            }
            .encode()?
            .as_slice(),
        )
        .await?;

    decode_and_print_message(socket).await?;

    Ok(())
}

async fn handle_queue(socket: UnixDatagram, sub_matches: &ArgMatches) -> HandleResult {
    let (work_time, break_time) = util::parse_work_and_break_time(sub_matches)?;

    socket
        .send(
            MessageRequest::Queue {
                work: work_time,
                r#break: break_time,
            }
            .encode()?
            .as_slice(),
        )
        .await?;

    decode_and_print_message(socket).await?;

    Ok(())
}

async fn handle_delete(socket: UnixDatagram, sub_matches: &ArgMatches) -> HandleResult {
    let (id, all) = if sub_matches.is_present("id") {
        (util::parse_arg::<u16>(sub_matches, "id")?, false)
    } else {
        (0, true)
    };

    socket
        .send(MessageRequest::Delete { id, all }.encode()?.as_slice())
        .await?;

    decode_and_print_message(socket).await?;

    Ok(())
}

async fn handle_list(socket: UnixDatagram) -> HandleResult {
    socket
        .send(MessageRequest::List.encode()?.as_slice())
        .await?;

    decode_and_print_message(socket).await?;

    Ok(())
}

async fn handle_test(socket: UnixDatagram) -> HandleResult {
    socket
        .send(MessageRequest::Test.encode()?.as_slice())
        .await?;

    decode_and_print_message(socket).await?;

    Ok(())
}

async fn handle_history(socket: UnixDatagram) -> HandleResult {
    socket
        .send(MessageRequest::History.encode()?.as_slice())
        .await?;

    decode_and_print_message(socket).await?;

    Ok(())
}

async fn decode_and_print_message(socket: UnixDatagram) -> HandleResult {
    let mut buf = vec![0u8; BUFFER_LENGTH];
    let (size, _) = socket.recv_from(&mut buf).await?;
    let dgram = &buf[..size];

    MessageResponse::decode(dgram)?.print();

    Ok(())
}
