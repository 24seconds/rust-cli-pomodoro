use chrono::{prelude::*, Duration};
use gluesql::{ast::DataType, Value};
use notify_rust::{error::Error, Hint, Notification as NR_Notification, Timeout as NR_Timeout};
use tabled::Tabled;

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

        Notification {
            id,
            created_at: utc,
            description: String::from("sample"),
            work_expired_at: utc + Duration::minutes(work_time as i64),
            break_expired_at: utc + Duration::minutes(break_time as i64),
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
            let work_min = sec / 60;
            let work_sec = sec - work_min * 60;

            String::from(format!("{}:{}", work_min, work_sec))
        };

        let break_remaining = {
            let sec = (self.break_expired_at - utc).num_seconds();
            let break_min = sec / 60;
            let break_sec = sec - break_min * 60;

            String::from(format!("{}:{}", break_min, break_sec))
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

pub fn notify_work() -> Result<(), Error> {
    NR_Notification::new()
        .summary("Work time done!")
        .body("Work time finished.\nNow take a rest!")
        .appname("pomodoro")
        .hint(Hint::Category("im.received".to_owned()))
        .timeout(NR_Timeout::Milliseconds(5000))
        .show()?;

    Ok(())
}

pub fn notify_break() -> Result<(), Error> {
    NR_Notification::new()
        .summary("Break time done!")
        .body("Break time finished.\n Now back to work!")
        .appname("pomodoro")
        .hint(Hint::Category("im.received".to_owned()))
        .timeout(NR_Timeout::Milliseconds(5000))
        .show()?;

    Ok(())
}
