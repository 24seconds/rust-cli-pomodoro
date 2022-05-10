use bincode;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use tokio::net::UnixDatagram;

use crate::command::ActionType;
use crate::InputSource;
use crate::UserInput;

const SOCKET_SERVER_ADDR: &str = "rust-cli-pomodoro-server.sock";
const SOCKET_CLIENT_ADDR: &str = "rust-cli-pomodoro-client.sock";

pub enum UdsType {
    Server,
    Client,
}

#[derive(bincode::Encode, bincode::Decode, PartialEq, Debug, Eq)]
pub enum MessageRequest {
    Create { work: u16, r#break: u16 },
    Queue { work: u16, r#break: u16 },
    Delete { id: u16, all: bool },
    List,
    Test,
    History,
}

impl MessageRequest {
    // TODO(young): handle error
    pub fn encode(self) -> Result<Vec<u8>, Box<dyn Error>> {
        let vec = bincode::encode_to_vec(self, bincode::config::standard())?;

        Ok(vec)
    }

    // TODO(young): handle error
    pub fn decode(byte: &[u8]) -> Result<Self, Box<dyn Error>> {
        let (request, _): (MessageRequest, usize) =
            bincode::decode_from_slice(byte, bincode::config::standard())?;

        Ok(request)
    }
}

impl From<MessageRequest> for UserInput {
    fn from(request: MessageRequest) -> Self {
        let input = match request {
            MessageRequest::Create { work, r#break } => {
                format!(
                    "{} -w {} -b {}",
                    String::from(ActionType::CREATE),
                    work,
                    r#break
                )
            }
            MessageRequest::Queue { work, r#break } => {
                format!(
                    "{} -w {} -b {}",
                    String::from(ActionType::QUEUE),
                    work,
                    r#break
                )
            }
            MessageRequest::Delete { id, all } => {
                // TODO(young): use match instead of if?
                if all {
                    format!("{} -a", String::from(ActionType::DELETE))
                } else {
                    format!("{} -id {}", String::from(ActionType::DELETE), id)
                }
            }
            MessageRequest::List => String::from(ActionType::LIST),
            MessageRequest::Test => String::from(ActionType::TEST),
            MessageRequest::History => String::from(ActionType::HISTORY),
        };

        UserInput {
            input,
            source: InputSource::UnixDomainSocket,
        }
    }
}

#[derive(bincode::Encode, bincode::Decode, PartialEq, Debug, Eq)]
pub struct MessageResponse {
    body: Vec<String>,
}

impl MessageResponse {
    pub fn new(body: Vec<String>) -> Self {
        MessageResponse { body }
    }

    pub fn get_body(&self) -> &Vec<String> {
        &self.body
    }

    // TODO(young): handle error
    pub fn encode(self) -> Result<Vec<u8>, Box<dyn Error>> {
        let vec = bincode::encode_to_vec(self, bincode::config::standard())?;

        Ok(vec)
    }

    // TODO(young): handle error
    pub fn decode(byte: &[u8]) -> Result<Self, Box<dyn Error>> {
        let (response, _): (MessageResponse, usize) =
            bincode::decode_from_slice(byte, bincode::config::standard())?;

        Ok(response)
    }

    pub fn print(self) {
        self.get_body().iter().for_each(|m| println!("{}", m));
    }
}

// TODO(young): handle error type precisely instead of using dyn Error
pub fn create_server_uds() -> Result<UnixDatagram, Box<dyn Error>> {
    // TODO(young): handle create_uds_address error
    let server_addr = create_uds_address(UdsType::Server, true)?;
    let socket = UnixDatagram::bind(server_addr)?;

    debug!("create_server_uds called");
    Ok(socket)
}

// TODO(young): handle unixdatagram error
// TODO(young): handle create_uds_address error
pub async fn create_client_uds() -> Result<UnixDatagram, Box<dyn Error>> {
    let server_addr = create_uds_address(UdsType::Server, false)?;
    let client_addr = create_uds_address(UdsType::Client, true)?;

    let socket = UnixDatagram::bind(client_addr)?;
    let _ = socket.connect(server_addr)?;

    debug!("create_client_uds called");
    Ok(socket)
}

fn create_uds_address(r#type: UdsType, should_remove: bool) -> std::io::Result<PathBuf> {
    let path = get_uds_address(r#type);

    if should_remove && path.exists() {
        debug!("patt {:?} exists, remove it before binding", &path);
        fs::remove_file(&path)?;
    }

    debug!("create_uds_address, path: {:?}", path);

    Ok(path)
}

pub fn get_uds_address(r#type: UdsType) -> PathBuf {
    let socket_addr = match r#type {
        UdsType::Server => SOCKET_SERVER_ADDR,
        UdsType::Client => SOCKET_CLIENT_ADDR,
    };

    let mut p = env::temp_dir();
    p.push(socket_addr);

    p
}
