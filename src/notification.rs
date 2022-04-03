use chrono::{prelude::*, Duration};
use gluesql::core::data::Value;
use notify_rust::{
    error::Error as NotifyRustError, Hint, Notification as NR_Notification, Timeout as NR_Timeout,
};
use reqwest::Error as RequestError;
use serde_json::json;
use std::{fmt, result};
use tokio::join;

#[cfg(target_os = "macos")]
use std::process::Command;
use std::sync::Arc;
use tabled::Tabled;

use crate::configuration::{Configuration, SLACK_API_URL};

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

/// The notification schema used to store to database
#[derive(Debug)]
pub struct Notification {
    id: u16,
    description: String,
    work_time: u16,
    break_time: u16,
    created_at: DateTime<Utc>,
    work_expired_at: DateTime<Utc>,
    break_expired_at: DateTime<Utc>,
}

impl<'a> Notification {
    pub fn new(id: u16, work_time: u16, break_time: u16, created_at: DateTime<Utc>) -> Self {
        let work_expired_at = created_at + Duration::minutes(work_time as i64);
        let break_expired_at = work_expired_at + Duration::minutes(break_time as i64);

        Notification {
            id,
            description: String::from("sample"),
            work_time,
            break_time,
            created_at,
            work_expired_at,
            break_expired_at,
        }
    }

    pub fn get_id(&self) -> u16 {
        self.id
    }

    pub fn get_start_at(&self) -> DateTime<Utc> {
        let last_expired_at = self.work_expired_at.max(self.break_expired_at);
        let duration = Duration::minutes((self.work_time + self.break_time) as i64);

        last_expired_at - duration
    }

    pub fn get_values(
        &'a self,
    ) -> (
        u16,
        &'a str,
        u16,
        u16,
        DateTime<Utc>,
        DateTime<Utc>,
        DateTime<Utc>,
    ) {
        (
            self.id,
            self.description.as_str(),
            self.work_time,
            self.break_time,
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

        let work_time = match &row.get(2).unwrap() {
            Value::I64(t) => *t as u16,
            _ => {
                panic!("notification work_time type mismatch")
            }
        };

        let break_time = match &row.get(3).unwrap() {
            Value::I64(t) => *t as u16,
            _ => {
                panic!("notification break_time type mismatch")
            }
        };

        let created_at = match &row.get(4).unwrap() {
            Value::Timestamp(t) => Utc.from_local_datetime(t).unwrap(),
            _ => {
                panic!("notification created_at type mismatch");
            }
        };

        let work_expired_at = match &row.get(5).unwrap() {
            Value::Timestamp(t) => Utc.from_local_datetime(t).unwrap(),
            _ => {
                panic!("notification work_expired_at type mismatch");
            }
        };

        let break_expired_at = match &row.get(6).unwrap() {
            Value::Timestamp(t) => Utc.from_local_datetime(t).unwrap(),
            _ => {
                panic!("notification break_expired_at type mismatch");
            }
        };

        Notification {
            id,
            description,
            work_time,
            break_time,
            created_at,
            work_expired_at,
            break_expired_at,
        }
    }
}

impl Tabled for Notification {
    const LENGTH: usize = 7;

    fn fields(&self) -> Vec<String> {
        let utc = Utc::now();

        let id = self.id.to_string();

        let work_remaining = if self.work_time > 0 {
            let sec = (self.work_expired_at - utc).num_seconds();

            if sec > 0 {
                let work_min = sec / 60;
                let work_sec = sec - work_min * 60;

                format!("{}:{}", work_min, work_sec)
            } else {
                String::from("00:00")
            }
        } else {
            String::from("N/A")
        };

        let break_remaining = if self.break_time > 0 {
            let sec = (self.break_expired_at - utc).num_seconds();

            if sec > 0 {
                let break_min = sec / 60;
                let break_sec = sec - break_min * 60;

                format!("{}:{}", break_min, break_sec)
            } else {
                String::from("00:00")
            }
        } else {
            String::from("N/A")
        };

        let start_at = {
            let local_time: DateTime<Local> = self.get_start_at().into();
            local_time.format("%F %T %z").to_string()
        };

        let description = self.description.to_string();

        let work_expired_at = if self.work_time > 0 {
            let local_time: DateTime<Local> = self.work_expired_at.into();
            local_time.format("%F %T %z").to_string()
        } else {
            String::from("N/A")
        };

        let break_expired_at = if self.break_time > 0 {
            let local_time: DateTime<Local> = self.break_expired_at.into();
            local_time.format("%F %T %z").to_string()
        } else {
            String::from("N/A")
        };

        vec![
            id,
            work_remaining,
            break_remaining,
            start_at,
            work_expired_at,
            break_expired_at,
            description,
        ]
    }

    fn headers() -> Vec<String> {
        vec![
            "id",
            "work_remaining (min)",
            "break_remaining (min)",
            "start_at",
            "expired_at (work)",
            "expired_at (break)",
            "description",
        ]
        .into_iter()
        .map(|x| x.to_string())
        .collect()
    }
}

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
async fn notify_slack(
    message: &'static str,
    configuration: &Arc<Configuration>,
) -> result::Result<(), NotificationError> {
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

    resp.map(|_| ()).map_err(|e| NotificationError::Slack(e))
}

/// notify_discord send notification to discord
/// use discord webhook notification if configuration specified
async fn notify_discord(
    message: &'static str,
    configuration: &Arc<Configuration>,
) -> result::Result<(), NotificationError> {
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

    resp.map(|_| ()).map_err(|e| NotificationError::Discord(e))
}

/// notify_dekstop send notification to desktop.
/// use notify-rust library for desktop notification
async fn notify_desktop(
    summary_message: &'static str,
    body_message: &'static str,
) -> result::Result<(), NotificationError> {
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
        .map_err(|e| NotificationError::Desktop(e))
}

pub async fn notify_work(configuration: &Arc<Configuration>) -> Result<(), NotificationError> {
    // TODO(young): Handle this also as async later
    #[cfg(target_os = "macos")]
    notify_terminal_notifier("work done. Take a rest!");

    let desktop_fut = notify_desktop("Work time done!", "Work time finished.\nNow take a rest!");
    let slack_fut = notify_slack("work done. Take a rest!", configuration);
    let discord_fut = notify_discord("work done. Take a rest!", configuration);

    // ??? check how tokio join works later
    let (desktop_result, slack_result, discord_result) = join!(desktop_fut, slack_fut, discord_fut);

    Ok(())
}

pub async fn notify_break(configuration: &Arc<Configuration>) -> Result<(), NotificationError> {
    #[cfg(target_os = "macos")]
    notify_terminal_notifier("break done. Get back to work");

    let desktop_fut = notify_desktop(
        "Break time done!",
        "Break time finished.\n Now back to work!",
    );
    let slack_fut = notify_slack("break done. Get back to work", configuration);
    let discord_fut = notify_discord("break done. Get back to work", configuration);

    // ??? check how tokio join works later
    let (desktop_result, slack_result, discord_result) = join!(desktop_fut, slack_fut, discord_fut);

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use gluesql::core::data::Value;
    use tabled::Tabled;

    use super::Notification;

    #[test]
    fn test_notification() {
        let now = Utc::now();
        let notification1 = Notification::new(0, 25, 5, now);
        assert_eq!(
            now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            notification1
                .get_start_at()
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        );

        let notification2 = {
            let naive_date_time = {
                let date = NaiveDate::from_ymd(2022, 3, 27);
                let time = NaiveTime::from_hms_milli(16, 08, 56, 789);

                NaiveDateTime::new(date, time)
            };
            let row = vec![
                Value::I64(0),
                Value::Str("sample".to_string()),
                Value::I64(25),
                Value::I64(5),
                Value::Timestamp(naive_date_time),
                Value::Timestamp(naive_date_time + Duration::minutes(25)),
                Value::Timestamp(naive_date_time + Duration::minutes(30)),
            ];

            Notification::convert_to_notification(row)
        };

        let test_cases = vec![
            ("notification::new test", notification1),
            ("notification::convert_to_notification test", notification2),
        ];

        test_cases.into_iter().for_each(|pair| {
            let (test_case, notification) = pair;

            assert_eq!(0, notification.get_id());
            let (id, desc, wt, bt, _, w_expired_at, b_expired_at) = notification.get_values();
            assert_eq!(0, id, "failed: {}", test_case);
            assert_eq!("sample", desc, "failed: {}", test_case);
            assert_eq!(25, wt, "failed: {}", test_case);
            assert_eq!(5, bt, "failed: {}", test_case);

            let start_at = notification.get_start_at();

            let expected_w_expired_at = start_at + Duration::minutes(25);
            assert_eq!(
                expected_w_expired_at.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                w_expired_at.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "failed: {}",
                test_case
            );

            let expected_b_expired_at = expected_w_expired_at + Duration::minutes(5);
            assert_eq!(
                expected_b_expired_at.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                b_expired_at.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "failed: {}",
                test_case
            );
        });
    }

    #[test]
    fn test_notification_tabled_impl() {
        let now = Utc::now();
        let notification = Notification::new(0, 25, 5, now);

        let fields = notification.fields();
        assert_eq!(7, fields.len());

        let headers = Notification::headers();
        assert_eq!(7, headers.len());
        assert_eq!(
            vec![
                "id".to_string(),
                "work_remaining (min)".to_string(),
                "break_remaining (min)".to_string(),
                "start_at".to_string(),
                "expired_at (work)".to_string(),
                "expired_at (break)".to_string(),
                "description".to_string(),
            ],
            headers
        );
    }
}
