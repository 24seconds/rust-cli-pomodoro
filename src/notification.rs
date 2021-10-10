use chrono::{prelude::*, Duration};

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
}
