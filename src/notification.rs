use chrono::{prelude::*, Duration};
use notify_rust::{error::Error, Hint, Notification as NR_Notification, Timeout as NR_Timeout};

pub struct Notification {
    id: u16,
    description: &'static str,
    created_at: DateTime<Utc>,
    work_expired_at: DateTime<Utc>,
    break_expired_at: DateTime<Utc>,
}

impl Notification {
    pub fn new(id: u16, work_time: u16, break_time: u16) -> Self {
        let utc = Utc::now();

        Notification {
            id,
            created_at: utc,
            description: "sample",
            work_expired_at: utc + Duration::minutes(work_time as i64),
            break_expired_at: utc + Duration::minutes(break_time as i64),
        }
    }

    pub fn get_values(
        &self,
    ) -> (
        u16,
        &'static str,
        DateTime<Utc>,
        DateTime<Utc>,
        DateTime<Utc>,
    ) {
        (
            self.id,
            self.description,
            self.created_at,
            self.work_expired_at,
            self.break_expired_at,
        )
    }

    pub fn get_id(&self) -> u16 {
        self.id
    }

pub fn notify() -> Result<(), Error> {
    NR_Notification::new()
        .summary("Work time done!")
        .body("This has nothing to do with emails.\nIt should not go away until you acknowledge it.")
        .icon("chrome")
        // .appname("thunderbird")
        .hint(Hint::Category("im.received".to_owned()))
        .hint(Hint::Resident(true)) // this is not supported by all implementations
        // .timeout(Timeout(5000)) // this however is
        .show()?;

    Ok(())
}
