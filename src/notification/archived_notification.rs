use chrono::{prelude::*, Duration};
use tabled::Tabled;

use crate::notification::Notification;

#[derive(Debug)]
pub struct ArchivedNotification {
    id: u16,
    description: String,
    work_time: u16,
    break_time: u16,
    // TODO(young): Maybe we don't need created_at field for ArchivedNotification
    created_at: DateTime<Utc>,
    work_expired_at: DateTime<Utc>,
    break_expired_at: DateTime<Utc>,
}

impl From<Notification> for ArchivedNotification {
    fn from(n: Notification) -> Self {
        let (id, desc, wt, bt, created_at, w_expired_at, b_expired_at) = n.get_values();

        ArchivedNotification {
            id,
            description: desc.to_string(),
            work_time: wt,
            break_time: bt,
            created_at,
            work_expired_at: w_expired_at,
            break_expired_at: b_expired_at,
        }
    }
}

impl ArchivedNotification {
    pub fn get_start_at(&self) -> DateTime<Utc> {
        let last_expired_at = self.work_expired_at.max(self.break_expired_at);
        let duration = Duration::minutes((self.work_time + self.break_time) as i64);

        last_expired_at - duration
    }
}

impl Tabled for ArchivedNotification {
    const LENGTH: usize = 7;

    fn fields(&self) -> Vec<String> {
        let id = self.id.to_string();

        let started_at = {
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
            self.work_time.to_string(),
            self.break_time.to_string(),
            started_at,
            work_expired_at,
            break_expired_at,
            description,
        ]
    }

    fn headers() -> Vec<String> {
        vec![
            "id",
            "work_time",
            "break_time",
            "started_at",
            "expired_at (work)",
            "expired_at (break)",
            "description",
        ]
        .into_iter()
        .map(|x| x.to_string())
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::notification::Notification;
    use chrono::Utc;
    use tabled::Tabled;

    use super::ArchivedNotification;

    #[test]
    fn test_archived_notification_conversion() {
        let now = Utc::now();
        let notification = Notification::new(0, 25, 5, now);
        let archived_notification = ArchivedNotification::from(notification);

        assert_eq!(
            now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            archived_notification
                .get_start_at()
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        );
    }

    #[test]
    fn test_archived_notification_tabled_impl() {
        let now = Utc::now();
        let notification = Notification::new(0, 25, 5, now);
        let archived_notification = ArchivedNotification::from(notification);

        let fields = archived_notification.fields();
        assert_eq!(7, fields.len());

        let headers = ArchivedNotification::headers();
        assert_eq!(7, headers.len());
        assert_eq!(
            vec![
                "id".to_string(),
                "work_time".to_string(),
                "break_time".to_string(),
                "started_at".to_string(),
                "expired_at (work)".to_string(),
                "expired_at (break)".to_string(),
                "description".to_string(),
            ],
            headers
        );
    }
}
