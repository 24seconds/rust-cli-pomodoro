use notify_rust::error::Error as NotifyRustError;
use reqwest::Error as RequestError;
use std::fmt;

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
            NotificationError::Desktop(_) => write!(f, "erorr while sending desktop notification"),
            NotificationError::Slack(_) => write!(f, "error while sending slack notification"),
            NotificationError::Discord(_) => write!(f, "error while sending slack notification"),
            NotificationError::EmptyConfiguration => {
                write!(f, "error while sending slack notification")
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
