use notify_rust::error::Error as NotifyRustError;
use reqwest::Error as RequestError;
use serde_json::error::Error as SerdeJsonError;
use std::{fmt, io, result};

pub type NotifyResult = result::Result<(), NotificationError>;

// notification error enum
#[derive(Debug)]
pub enum NotificationError {
    // TODO(Desktop also need NotifyRustError type???)
    Desktop(NotifyRustError),
    Slack(RequestError),
    Discord(RequestError),
    EmptyConfiguration,
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotificationError::Desktop(_) => write!(f, "NotificationError::Desktop"),
            NotificationError::Slack(_) => write!(f, "NotificationError::Slack"),
            NotificationError::Discord(_) => write!(f, "NotificationError::Discord"),
            NotificationError::EmptyConfiguration => {
                write!(f, "configuration is empty")
            }
        }
    }
}

impl std::error::Error for NotificationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NotificationError::Desktop(ref e) => Some(e),
            NotificationError::Slack(ref e) => Some(e),
            NotificationError::Discord(ref e) => Some(e),
            NotificationError::EmptyConfiguration => None,
        }
    }
}

#[derive(Debug)]
pub enum ConfigurationError {
    FileNotFound,
    FileOpenError(io::Error),
    JsonError(SerdeJsonError),
    SlackConfigNotFound,
    DiscordConfigNotFound,
    // config json wrong format?
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigurationError::FileNotFound => write!(f, "can not find configuration file "),
            ConfigurationError::FileOpenError(_) => write!(f, "failed to open the file"),
            ConfigurationError::JsonError(_) => write!(f, "failed to deserialize json"),
            ConfigurationError::SlackConfigNotFound => {
                write!(f, "can not find slack config in json")
            }
            ConfigurationError::DiscordConfigNotFound => {
                write!(f, "can not find discord config in json")
            }
        }
    }
}

impl std::error::Error for ConfigurationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigurationError::FileNotFound => None,
            ConfigurationError::FileOpenError(ref e) => Some(e),
            ConfigurationError::JsonError(ref e) => Some(e),
            ConfigurationError::SlackConfigNotFound => None,
            ConfigurationError::DiscordConfigNotFound => None,
        }
    }
}
