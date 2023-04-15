use crate::ipc::{Bincodec, UdsMessage};
use clap::ArgMatches;
use std::result;
use tokio::net::UnixDatagram;

use crate::command::action::ActionType;
use crate::command::util;
use crate::error::UdsHandlerError;
use crate::ipc::{MessageRequest, MessageResponse};

pub const BUFFER_LENGTH: usize = 100_000_000;

type HandleUdsResult = result::Result<(), UdsHandlerError>;

// TODO(young): handle error properly
pub async fn handle(matches: ArgMatches, socket: UnixDatagram) -> HandleUdsResult {
    let (action_type, sub_matches) = matches
        .subcommand()
        .ok_or(UdsHandlerError::NoSubcommand)
        .and_then(|(s, sub_matches)| {
            ActionType::parse(s)
                .map(|s| (s, sub_matches))
                .map_err(UdsHandlerError::ParseError)
        })?;

    match action_type {
        ActionType::Create => handle_create(socket, sub_matches).await?,
        ActionType::Queue => handle_queue(socket, sub_matches).await?,
        ActionType::Delete => handle_delete(socket, sub_matches).await?,
        ActionType::List => handle_list(socket, sub_matches).await?,
        ActionType::Test => handle_test(socket).await?,
        ActionType::History => handle_history(socket).await?,
        ActionType::Exit | ActionType::Clear => {
            info!("Exit or Clear is not supported action for unix domain client")
        }
    }

    Ok(())
}

async fn handle_create(socket: UnixDatagram, sub_matches: &ArgMatches) -> HandleUdsResult {
    let (work_time, break_time) =
        util::parse_work_and_break_time(sub_matches).map_err(UdsHandlerError::ParseError)?;

    socket
        .send(
            UdsMessage::Public(MessageRequest::Create {
                work: work_time,
                r#break: break_time,
            })
            .encode()
            .map_err(UdsHandlerError::EncodeFailed)?
            .as_slice(),
        )
        .await
        .map_err(UdsHandlerError::SocketError)?;

    decode_and_print_message(socket).await?;

    Ok(())
}

async fn handle_queue(socket: UnixDatagram, sub_matches: &ArgMatches) -> HandleUdsResult {
    let (work_time, break_time) =
        util::parse_work_and_break_time(sub_matches).map_err(UdsHandlerError::ParseError)?;

    debug!("handle_queue");
    socket
        .send(
            UdsMessage::Public(MessageRequest::Queue {
                work: work_time,
                r#break: break_time,
            })
            .encode()
            .map_err(UdsHandlerError::EncodeFailed)?
            .as_slice(),
        )
        .await
        .map_err(UdsHandlerError::SocketError)?;

    decode_and_print_message(socket).await?;

    Ok(())
}

async fn handle_delete(socket: UnixDatagram, sub_matches: &ArgMatches) -> HandleUdsResult {
    let (id, all) = if sub_matches.contains_id("id") {
        (
            util::parse_arg::<u16>(sub_matches, "id").map_err(UdsHandlerError::ParseError)?,
            false,
        )
    } else {
        (0, true)
    };

    socket
        .send(
            UdsMessage::Public(MessageRequest::Delete { id, all })
                .encode()
                .map_err(UdsHandlerError::EncodeFailed)?
                .as_slice(),
        )
        .await
        .map_err(UdsHandlerError::SocketError)?;

    decode_and_print_message(socket).await?;

    Ok(())
}

async fn handle_list(socket: UnixDatagram, sub_matches: &ArgMatches) -> HandleUdsResult {
    let show_percentage = sub_matches.get_flag("percentage");
    
    socket
        .send(
            UdsMessage::Public(MessageRequest::List {
                show_percentage,
            })
                .encode()
                .map_err(UdsHandlerError::EncodeFailed)?
                .as_slice(),
        )
        .await
        .map_err(UdsHandlerError::SocketError)?;

    decode_and_print_message(socket).await?;

    Ok(())
}

async fn handle_test(socket: UnixDatagram) -> HandleUdsResult {
    socket
        .send(
            UdsMessage::Public(MessageRequest::Test)
                .encode()
                .map_err(UdsHandlerError::EncodeFailed)?
                .as_slice(),
        )
        .await
        .map_err(UdsHandlerError::SocketError)?;

    decode_and_print_message(socket).await?;

    Ok(())
}

async fn handle_history(socket: UnixDatagram) -> HandleUdsResult {
    socket
        .send(
            UdsMessage::Public(MessageRequest::History)
                .encode()
                .map_err(UdsHandlerError::EncodeFailed)?
                .as_slice(),
        )
        .await
        .map_err(UdsHandlerError::SocketError)?;

    decode_and_print_message(socket).await?;

    Ok(())
}

async fn decode_and_print_message(socket: UnixDatagram) -> HandleUdsResult {
    let mut vec = Vec::new();
    let mut total_size = 0;

    // TODO(young): set timeout to prevent infinite loop
    loop {
        let mut buf = vec![0u8; BUFFER_LENGTH];
        let (size, _) = socket
            .recv_from(&mut buf)
            .await
            .map_err(UdsHandlerError::SocketError)?;
        debug!("decode_and_print_message, size: {}", size);

        let dgram = &buf[..size];
        debug!("dgram len: {:?}", dgram.len());
        vec.extend_from_slice(dgram);
        debug!("vec length: {:?}", vec.len());

        total_size += size;

        if size == 0 {
            break;
        }
    }

    debug!("total_size: {}", total_size);
    let dgram = &vec.as_slice()[..total_size];
    MessageResponse::decode(dgram)
        .map_err(UdsHandlerError::DecodeFailed)?
        .print();

    Ok(())
}
