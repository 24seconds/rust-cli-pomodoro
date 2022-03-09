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
    #[serde(rename(deserialize = "slack"))]
    slack_configuration: Option<SlackConfiguration>,
    #[serde(rename(deserialize = "discord"))]
    discord_configuration: Option<DiscordConfiguration>,
}

#[derive(Deserialize, Debug, Default)]
struct SlackConfiguration {
    token: Option<String>,
    channel: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
struct DiscordConfiguration {
    webhook_url: Option<String>,
}

impl Configuration {
    pub fn get_slack_token(&self) -> &Option<String> {
        match &self.slack_configuration {
            Some(config) => &config.token,
            None => &None,
        }
    }

    pub fn get_slack_channel(&self) -> &Option<String> {
        match &self.slack_configuration {
            Some(config) => &config.channel,
            None => &None,
        }
    }

    pub fn get_discord_webhook_url(&self) -> &Option<String> {
        match &self.discord_configuration {
            Some(config) => &config.webhook_url,
            None => &None,
        }
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
