use serde::Deserialize;

use std::env::{self, VarError};
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

const SLACK_TOKEN: &str = "SLACK_TOKEN";
const SLACK_CHANNEL: &str = "SLACK_CHANNEL";
pub const SLACK_API_URL: &str = "https://slack.com/api/chat.postMessage";
const CREDENTIAL_FILE: &str = "credential.json";

#[derive(Deserialize, Debug, Default)]
pub struct Configuration {
    #[serde(rename(deserialize = "token"))]
    slack_token: Option<String>,
    #[serde(rename(deserialize = "channel"))]
    slack_channel: Option<String>,
}

impl Configuration {
    fn set_slack_token(mut self, token: Option<String>) -> Self {
        self.slack_token = token;

        self
    }

    pub fn get_slack_token(&self) -> &Option<String> {
        &self.slack_token
    }

    fn set_slack_channel(mut self, channel: Option<String>) -> Self {
        self.slack_channel = channel;

        self
    }

    pub fn get_slack_channel(&self) -> &Option<String> {
        &self.slack_channel
    }
}

fn get_value_from_env(key: &str) -> Result<Option<String>, VarError> {
    let value = env::var(key).map_or_else(
        |e| match e {
            VarError::NotPresent => Ok(None),
            _ => Err(e),
        },
        |v| Ok(Some(v)),
    )?;

    Ok(value)
}

pub fn initialize_configuration() -> Result<Configuration, Box<dyn Error>> {
    let token = get_value_from_env(SLACK_TOKEN)?;
    let channel = get_value_from_env(SLACK_CHANNEL)?;

    let configuration = if token.is_some() && channel.is_some() {
        Configuration::default()
            .set_slack_token(token)
            .set_slack_channel(channel)
    } else {
        let path = env::current_dir()?.join(CREDENTIAL_FILE);

        read_credential_from_file(path)?
    };

    debug!("configuration: {:?}", configuration);

    Ok(configuration)
}

fn read_credential_from_file<P: AsRef<Path> + AsRef<OsStr>>(
    path: P,
) -> Result<Configuration, Box<dyn Error>> {
    if !Path::new(&path).exists() {
        return Ok(Configuration::default());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let c = serde_json::from_reader(reader)?;
    Ok(c)
}
