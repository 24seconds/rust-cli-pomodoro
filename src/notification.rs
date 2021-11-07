use chrono::{prelude::*, Duration};
use gluesql::Value;
use notify_rust::{error::Error, Hint, Notification as NR_Notification, Timeout as NR_Timeout};
use serde_json::json;
use std::process::Command;
use std::sync::Arc;
use tabled::Tabled;

use crate::configuration::{Configuration, SLACK_API_URL};

pub struct Notification {
    id: u16,
    description: String,
    created_at: DateTime<Utc>,
    work_expired_at: DateTime<Utc>,
    break_expired_at: DateTime<Utc>,
}

impl<'a> Notification {
    pub fn new(id: u16, work_time: u16, break_time: u16) -> Self {
        let utc = Utc::now();
        let work_expired_at = utc + Duration::minutes(work_time as i64);
        let break_expired_at = work_expired_at + Duration::minutes(break_time as i64);

        Notification {
            id,
            created_at: utc,
            description: String::from("sample"),
            work_expired_at,
            break_expired_at,
        }
    }

    pub fn get_values(&'a self) -> (u16, &'a str, DateTime<Utc>, DateTime<Utc>, DateTime<Utc>) {
        (
            self.id,
            self.description.as_str(),
            self.created_at,
            self.work_expired_at,
            self.break_expired_at,
        )
    }

    pub fn convert_to_notification(row: Vec<Value>) -> Self {
        let id = match &row.get(0).unwrap() {
            Value::I64(id) => *id as u16,
            _ => {
                panic!("notification id type mismatch");
            }
        };

        let description = match &row.get(1).unwrap() {
            Value::Str(s) => s.to_owned(),
            _ => {
                panic!("notification description type mismatch");
            }
        };

        let created_at = match &row.get(2).unwrap() {
            Value::Timestamp(t) => Utc.from_local_datetime(t).unwrap(),
            _ => {
                panic!("notification created_at type mismatch");
            }
        };

        let work_expired_at = match &row.get(3).unwrap() {
            Value::Timestamp(t) => Utc.from_local_datetime(t).unwrap(),
            _ => {
                panic!("notification work_expired_at type mismatch");
            }
        };

        let break_expired_at = match &row.get(4).unwrap() {
            Value::Timestamp(t) => Utc.from_local_datetime(t).unwrap(),
            _ => {
                panic!("notification break_expired_at type mismatch");
            }
        };

        Notification {
            id,
            description,
            created_at,
            work_expired_at,
            break_expired_at,
        }
    }
}

impl Tabled for Notification {
    fn fields(&self) -> Vec<String> {
        let utc = Utc::now();

        let id = self.id.to_string();

        let work_remaining = {
            let sec = (self.work_expired_at - utc).num_seconds();

            if sec > 0 {
                let work_min = sec / 60;
                let work_sec = sec - work_min * 60;

                format!("{}:{}", work_min, work_sec)
            } else {
                String::from("00:00")
            }
        };

        let break_remaining = {
            let sec = (self.break_expired_at - utc).num_seconds();

            if sec > 0 {
                let break_min = sec / 60;
                let break_sec = sec - break_min * 60;

                format!("{}:{}", break_min, break_sec)
            } else {
                String::from("00:00")
            }
        };

        let local_time: DateTime<Local> = utc.into();
        let created_at = local_time.format("%F %T %z").to_string();

        let description = self.description.to_string();

        vec![id, work_remaining, break_remaining, created_at, description]
    }

    fn headers() -> Vec<String> {
        vec![
            "id",
            "work_remaining (min)",
            "break_remaining (min)",
            "created_at",
            "description",
        ]
        .into_iter()
        .map(|x| x.to_string())
        .collect()
    }
}

// TODO(young): Handle the case if terminal notifier is not installed
fn notify_terminal_notifier(message: &'static str) {
    Command::new("terminal-notifier")
        .arg("-message")
        .arg(message)
        .output()
        .expect("failed to execute process");
}

async fn notify_slack(message: &'static str, configuration: &Arc<Configuration>) {
    let token = configuration.get_slack_token();
    let channel = configuration.get_slack_channel();

    if token.is_none() || channel.is_none() {
        debug!("token or channel is none");
        return;
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
}

pub async fn notify_work(configuration: &Arc<Configuration>) -> Result<(), Error> {
    let mut notification = NR_Notification::new();
    let notification = notification
        .summary("Work time done!")
        .body("Work time finished.\nNow take a rest!")
        .appname("pomodoro")
        .timeout(NR_Timeout::Milliseconds(5000));

    #[cfg(target_os = "linux")]
    notification.hint(Hint::Category("im.received".to_owned()));

    notification.show()?;

    #[cfg(target_os = "macos")]
    notify_terminal_notifier("work done. Take a rest!");

    // use slack notification if configuration specified
    notify_slack("work done. Take a rest!", configuration).await;

    Ok(())
}

pub async fn notify_break(configuration: &Arc<Configuration>) -> Result<(), Error> {
    let mut notification = NR_Notification::new();
    let notification = notification
        .summary("Break time done!")
        .body("Break time finished.\n Now back to work!")
        .appname("pomodoro")
        .timeout(NR_Timeout::Milliseconds(5000));

    #[cfg(target_os = "linux")]
    notification.hint(Hint::Category("im.received".to_owned()));

    notification.show()?;

    // use terminal-notifier for desktop notification
    #[cfg(target_os = "macos")]
    notify_terminal_notifier("break done. Get back to work");

    // use slack notification if configuration specified
    notify_slack("break done. Get back to work", configuration).await;

    Ok(())
}
