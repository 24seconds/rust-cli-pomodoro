use chrono::{prelude::*, Duration};
use gluesql::core::data::Value;
use notify_rust::{error::Error, Hint, Notification as NR_Notification, Timeout as NR_Timeout};
use serde_json::json;
use tokio::join;

#[cfg(target_os = "macos")]
use std::process::Command;
use std::sync::Arc;
use tabled::Tabled;

use crate::configuration::{Configuration, SLACK_API_URL};

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

// TODO(young): Handle the case if terminal notifier is not installed
#[cfg(target_os = "macos")]
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

async fn notify_discord(message: &'static str, configuration: &Arc<Configuration>) {
    let webhook_url = match configuration.get_discord_webhook_url() {
        Some(url) => url,
        None => {
            debug!("webhook_url is none");
            return;
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
}

pub async fn notify_work(configuration: &Arc<Configuration>) -> Result<(), Error> {
    let mut notification = NR_Notification::new();
    let notification = notification
        .summary("Work time done!")
        .body("Work time finished.\nNow take a rest!")
        .appname("pomodoro")
        .timeout(NR_Timeout::Milliseconds(5000));

    #[cfg(target_os = "linux")]
    notification
        .hint(Hint::Category("im.received".to_owned()))
        .sound_name("message-new-instant");

    notification.show()?;

    #[cfg(target_os = "macos")]
    notify_terminal_notifier("work done. Take a rest!");

    // use slack notification if configuration specified
    let slack_fut = notify_slack("work done. Take a rest!", configuration);
    // use discord webhook notification if configuration specified
    let discord_fut = notify_discord("work done. Take a rest!", configuration);

    join!(slack_fut, discord_fut);

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
    let slack_fut = notify_slack("break done. Get back to work", configuration);

    // use discord webhook notification if configuration specified
    let discord_fut = notify_discord("break done. Get back to work", configuration);

    join!(slack_fut, discord_fut);

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

    #[tokio::test]
    async fn test_send_slack() {}
}
