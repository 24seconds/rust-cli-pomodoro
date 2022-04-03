use colored::{ColoredString, Colorize};
#[cfg(target_os = "linux")]
use notify_rust::Hint;
use notify_rust::{Notification as NR_Notification, Timeout as NR_Timeout};
use serde_json::json;
use std::error::Error;
use tabled::{Style, Table, Tabled};
use tokio::join;

#[cfg(target_os = "macos")]
use std::process::Command;
use std::sync::Arc;

use crate::configuration::{Configuration, SLACK_API_URL};
use crate::error::{NotificationError, NotifyResult};

#[cfg(target_os = "macos")]
fn notify_terminal_notifier(message: &'static str) {
    use std::io::ErrorKind;

    let result = Command::new("terminal-notifier")
        .arg("-message")
        .arg(message)
        .output();

    match result {
        Ok(_) => debug!("terminal notifier called"),
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                debug!("terminla notifier not found");
            } else {
                debug!("error while executing terminal notifier: {:?}", e);
            }
        }
    }
}

/// notify_slack send notification to slack
/// it uses slack notification if configuration specified
async fn notify_slack(message: &'static str, configuration: &Arc<Configuration>) -> NotifyResult {
    let token = configuration.get_slack_token();
    let channel = configuration.get_slack_channel();

    if token.is_none() || channel.is_none() {
        debug!("token or channel is none");
        return Err(NotificationError::EmptyConfiguration);
    }

    let body = json!({
        "channel": channel,
        "text": message
    })
    .to_string();

    let client = reqwest::Client::new();
    let resp = client
        .post(SLACK_API_URL)
        .header("Content-Type", "application/json")
        .header(
            "Authorization",
            format!("Bearer {}", token.clone().unwrap()),
        )
        .body(body)
        .send()
        .await;

    debug!("resp: {:?}", resp);

    resp.map(|_| ()).map_err(NotificationError::Slack)
}

/// notify_discord send notification to discord
/// use discord webhook notification if configuration specified
async fn notify_discord(message: &'static str, configuration: &Arc<Configuration>) -> NotifyResult {
    let webhook_url = match configuration.get_discord_webhook_url() {
        Some(url) => url,
        None => {
            debug!("webhook_url is none");
            return Err(NotificationError::EmptyConfiguration);
        }
    };

    let body = json!({ "content": message }).to_string();

    let client = reqwest::Client::new();
    let resp = client
        .post(webhook_url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await;

    debug!("resp: {:?}", resp);

    resp.map(|_| ()).map_err(NotificationError::Discord)
}

/// notify_dekstop send notification to desktop.
/// use notify-rust library for desktop notification
async fn notify_desktop(summary_message: &'static str, body_message: &'static str) -> NotifyResult {
    let mut notification = NR_Notification::new();
    let notification = notification
        .summary(summary_message)
        .body(body_message)
        .appname("pomodoro")
        .timeout(NR_Timeout::Milliseconds(5000));

    #[cfg(target_os = "linux")]
    notification
        .hint(Hint::Category("im.received".to_owned()))
        .sound_name("message-new-instant");

    notification
        .show()
        .map(|_| ())
        .map_err(NotificationError::Desktop)
}

pub async fn notify_work(configuration: &Arc<Configuration>) -> Result<String, NotificationError> {
    // TODO(young): Handle this also as async later
    #[cfg(target_os = "macos")]
    notify_terminal_notifier("work done. Take a rest!");

    let desktop_fut = notify_desktop("Work time done!", "Work time finished.\nNow take a rest!");
    let slack_fut = notify_slack("work done. Take a rest!", configuration);
    let discord_fut = notify_discord("work done. Take a rest!", configuration);

    let (desktop_result, slack_result, discord_result) = join!(desktop_fut, slack_fut, discord_fut);

    Ok(generate_notify_report(
        desktop_result,
        slack_result,
        discord_result,
    ))
}

pub async fn notify_break(configuration: &Arc<Configuration>) -> Result<String, NotificationError> {
    #[cfg(target_os = "macos")]
    notify_terminal_notifier("break done. Get back to work");

    let desktop_fut = notify_desktop(
        "Break time done!",
        "Break time finished.\n Now back to work!",
    );
    let slack_fut = notify_slack("break done. Get back to work", configuration);
    let discord_fut = notify_discord("break done. Get back to work", configuration);

    let (desktop_result, slack_result, discord_result) = join!(desktop_fut, slack_fut, discord_fut);

    Ok(generate_notify_report(
        desktop_result,
        slack_result,
        discord_result,
    ))
}

#[derive(Tabled)]
struct Report {
    ok: ColoredString,
    notification_type: String,
    reason: String,
}

impl Report {
    pub fn new(ok: &'static str, notification_type: &'static str) -> Self {
        Report {
            ok: ok.green(),
            notification_type: String::from(notification_type),
            reason: String::default(),
        }
    }

    pub fn update_reason(mut self, e: NotificationError) -> Self {
        let mut vec = vec![format!("{}", e)];
        if let Some(s) = e.source() {
            vec.push(s.to_string());
        }

        self.reason = vec.join("\n");

        self
    }
}

fn generate_notify_report(
    desktop: NotifyResult,
    slack: NotifyResult,
    discord: NotifyResult,
) -> String {
    let desktop_message = match desktop {
        Ok(_) => Report::new("O", "Desktop"),
        Err(e) => Report::new("X", "Desktop").update_reason(e),
    };

    let slack_message = match slack {
        Ok(_) => Report::new("O", "Slack"),
        Err(e) => Report::new("X", "Slack").update_reason(e),
    };

    let discord_message = match discord {
        Ok(_) => Report::new("O", "Discord"),
        Err(e) => Report::new("X", "Discord").update_reason(e),
    };

    Table::new(vec![desktop_message, slack_message, discord_message])
        .with(Style::modern())
        .to_string()
}
