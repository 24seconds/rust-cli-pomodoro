use colored::{ColoredString, Colorize};
use serde::Deserialize;
use tabled::{Style, Table, Tabled};

use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::error::ConfigurationError;

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
) -> Result<(Configuration, Option<ConfigurationError>), Box<dyn Error>> {
    let (configuration, error) = match credential_file {
        Some(f) => {
            let path = env::current_dir()?.join(f);

            match get_configuration_from_file(path) {
                Ok(config) => (config, None),
                Err(e) => (Configuration::default(), Some(e)),
            }
        }
        None => (Configuration::default(), None),
    };

    debug!("configuration: {:?}", configuration);

    Ok((configuration, error))
}

fn get_configuration_from_file<P: AsRef<Path> + AsRef<OsStr>>(
    path: P,
) -> Result<Configuration, ConfigurationError> {
    if !Path::new(&path).exists() {
        return Err(ConfigurationError::FileNotFound);
    }

    let file = File::open(path).map_err(ConfigurationError::FileOpenError)?;
    let reader = BufReader::new(file);

    let c = serde_json::from_reader(reader).map_err(ConfigurationError::JsonError)?;
    Ok(c)
}

#[derive(Tabled)]
struct Report {
    ok: ColoredString,
    desc: String,
    reason: ColoredString,
}

impl Report {
    pub fn new(ok: &'static str, desc: &'static str) -> Self {
        Report {
            ok: ok.green(),
            desc: String::from(desc),
            reason: ColoredString::default(),
        }
    }

    // TODO(young): e should be string or ConfigurationError?
    pub fn update_reason(mut self, e: ConfigurationError) -> Self {
        let mut vec = vec![format!("{}", e)];
        if let Some(s) = e.source() {
            vec.push(s.to_string());
        }

        self.reason = vec.join("\n").red();

        self
    }
}

pub fn generate_configuration_report(
    config: &Configuration,
    err: Option<ConfigurationError>,
) -> String {
    let config_err_message = match err {
        Some(e) => Report::new("X", "config err").update_reason(e),
        None => Report::new("O", "config err"),
    };

    let slack_channel_message = match config.get_slack_channel() {
        // TODO(young): Reason?
        Some(_) => Report::new("O", "slack_channel"),
        None => {
            Report::new("X", "slack_channel").update_reason(ConfigurationError::SlackConfigNotFound)
        }
    };

    let slack_token_message = match config.get_slack_channel() {
        // TODO(young): Reason?
        Some(_) => Report::new("O", "slack_token"),
        None => {
            Report::new("X", "slack_token").update_reason(ConfigurationError::SlackConfigNotFound)
        }
    };

    let discord_webhook_url_message = match config.get_discord_webhook_url() {
        // TODO(young): Reason?
        Some(_) => Report::new("O", "discord_webhook_url"),
        None => Report::new("X", "discord_webhook_url")
            .update_reason(ConfigurationError::DiscordConfigNotFound),
    };

    Table::new(vec![
        config_err_message,
        slack_channel_message,
        slack_token_message,
        discord_webhook_url_message,
    ])
    .with(Style::modern())
    .to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::initialize_configuration;

    #[test]
    fn test_initialize_configuration_some() {
        let file = PathBuf::from("resources/test/mock_credential.json");
        let file = file.to_str();

        let result = initialize_configuration(file);
        assert_eq!(true, result.is_ok());
        let config = result.unwrap().0;

        let slack_token = config.get_slack_token();
        assert_eq!(true, slack_token.is_some());
        assert!(slack_token.as_ref().unwrap().eq("your-bot-token-string"));

        let slack_channel = config.get_slack_channel();
        assert_eq!(true, slack_channel.is_some());
        assert!(slack_channel.as_ref().unwrap().eq("your-slack-channel-id"));

        let discord_webhook_url = config.get_discord_webhook_url();
        assert_eq!(true, discord_webhook_url.is_some());
        assert!(discord_webhook_url.as_ref().unwrap().eq("your-webhook-url"));
    }

    #[test]
    fn test_initialize_configuration_none() {
        [PathBuf::from("wrong_path").to_str(), None]
            .into_iter()
            .for_each(|file| {
                let result = initialize_configuration(file);
                assert_eq!(true, result.is_ok());
                let config = result.unwrap().0;

                let slack_token = config.get_slack_token();
                assert_eq!(true, slack_token.is_none());

                let slack_channel = config.get_slack_channel();
                assert_eq!(true, slack_channel.is_none());

                let discord_webhook_url = config.get_discord_webhook_url();
                assert_eq!(true, discord_webhook_url.is_none());
            });
    }
}
