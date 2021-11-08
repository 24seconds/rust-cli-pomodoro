use serde::Deserialize;

use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub const SLACK_API_URL: &str = "https://slack.com/api/chat.postMessage";

#[derive(Deserialize, Debug, Default)]
pub struct Configuration {
    #[serde(rename(deserialize = "token"))]
    slack_token: Option<String>,
    #[serde(rename(deserialize = "channel"))]
    slack_channel: Option<String>,
}

impl Configuration {
    pub fn get_slack_token(&self) -> &Option<String> {
        &self.slack_token
    }

    pub fn get_slack_channel(&self) -> &Option<String> {
        &self.slack_channel
    }
}

pub fn initialize_configuration(
    credential_file: Option<&str>,
) -> Result<Configuration, Box<dyn Error>> {
    let configuration = match credential_file {
        Some(f) => {
            let path = env::current_dir()?.join(f);

            get_configuration_from_file(path)?
        }
        None => Configuration::default(),
    };

    debug!("configuration: {:?}", configuration);

    Ok(configuration)
}

fn get_configuration_from_file<P: AsRef<Path> + AsRef<OsStr>>(
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
