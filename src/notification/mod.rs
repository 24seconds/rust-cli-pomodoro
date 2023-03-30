pub(crate) mod archived_notification;
pub(crate) mod notify;

use std::sync::Arc;

pub use archived_notification::*;
pub use notify::*;

use chrono::{prelude::*, Duration};
use clap::ArgMatches;
use gluesql::core::data::Value;
use gluesql::prelude::Row;
use tabled::Tabled;

use crate::command::util::parse_arg;
use crate::command::{DEFAULT_BREAK_TIME, DEFAULT_WORK_TIME};
use crate::configuration::Configuration;
use crate::db;
use crate::error::NotificationError;
use crate::{ArcGlue, ArcTaskMap};

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

    pub fn convert_to_notification(row: Row) -> Self {
        let id = match row.get_value_by_index(0).unwrap() {
            Value::I64(id) => *id as u16,
            _ => {
                panic!("notification id type mismatch");
            }
        };

        let description = match row.get_value_by_index(1).unwrap() {
            Value::Str(s) => s.to_owned(),
            _ => {
                panic!("notification description type mismatch");
            }
        };

        let work_time = match row.get_value_by_index(2).unwrap() {
            Value::I64(t) => *t as u16,
            _ => {
                panic!("notification work_time type mismatch")
            }
        };

        let break_time = match row.get_value_by_index(3).unwrap() {
            Value::I64(t) => *t as u16,
            _ => {
                panic!("notification break_time type mismatch")
            }
        };

        let created_at = match row.get_value_by_index(4).unwrap() {
            Value::Timestamp(t) => Utc.from_local_datetime(t).unwrap(),
            _ => {
                panic!("notification created_at type mismatch");
            }
        };

        let work_expired_at = match row.get_value_by_index(5).unwrap() {
            Value::Timestamp(t) => Utc.from_local_datetime(t).unwrap(),
            _ => {
                panic!("notification work_expired_at type mismatch");
            }
        };

        let break_expired_at = match row.get_value_by_index(6).unwrap() {
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

pub fn get_new_notification(
    matches: &ArgMatches,
    id_manager: &mut u16,
    created_at: DateTime<Utc>,
    configuration: Arc<Configuration>,
) -> Result<Notification, NotificationError> {
    let mut work_time = match configuration.get_work_time() {
        Some(work_time) => work_time,
        None => DEFAULT_WORK_TIME,
    };

    let mut break_time = match configuration.get_break_time() {
        Some(break_time) => break_time,
        None => DEFAULT_BREAK_TIME,
    };

    if matches.get_one::<String>("work").is_some() {
        println!("work is some");
        work_time =
            parse_arg::<u16>(matches, "work").map_err(NotificationError::NewNotification)?;
    }

    if matches.get_one::<String>("break").is_some() {
        println!("break is some");
        break_time =
            parse_arg::<u16>(matches, "break").map_err(NotificationError::NewNotification)?;
    }

    debug!("work_time: {}", work_time);
    debug!("break_time: {}", break_time);

    let id = get_new_id(id_manager);

    Ok(Notification::new(id, work_time, break_time, created_at))
}

fn get_new_id(id_manager: &mut u16) -> u16 {
    let id = *id_manager;
    *id_manager += 1;

    id
}

pub async fn delete_notification(
    id: u16,
    notification_task_map: ArcTaskMap,
    glue: ArcGlue,
) -> Result<(), NotificationError> {
    let notification = db::read_notification(glue.clone(), id).await;
    if notification.is_none() {
        return Err(NotificationError::DeletionFail(format!(
            "deleting id ({}) failed. Corresponding notification does not exist",
            id
        )));
    }

    {
        let mut hash_map = notification_task_map.lock().unwrap();

        hash_map
            .get(&id)
            .ok_or(format!("failed to corresponding task (id: {})", &id))
            .map_err(NotificationError::DeletionFail)?
            .abort();

        hash_map
            .remove(&id)
            .ok_or(format!("failed to remove id ({})", id))
            .map_err(NotificationError::DeletionFail)?;
    }

    db::delete_and_archive_notification(glue, id).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use clap::Command;
    use gluesql::core::data::Value;
    use tabled::Tabled;

    use crate::command::add_args_for_create_subcommand;
    use crate::configuration::load_configuration;

    use super::get_new_notification;
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
            ]
            .into();

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

    #[test]
    fn test_get_new_notification() {
        let cmd = Command::new("myapp");
        let matches = add_args_for_create_subcommand(cmd)
            .get_matches_from("myapp -w 25 -b 5".split_whitespace());
        let mut id_manager = 0;
        let now = Utc::now();

        let (configuration, _) =
            load_configuration(Some("./../../resources/test/mock_configuration.json")).unwrap();
        let notification =
            get_new_notification(&matches, &mut id_manager, now, Arc::new(configuration)).unwrap();

        let (id, _, wt, bt, created_at, _, _) = notification.get_values();
        assert_eq!(0, id);
        assert_eq!(25, wt);
        assert_eq!(5, bt);
        assert_eq!(now, created_at);
    }
}
