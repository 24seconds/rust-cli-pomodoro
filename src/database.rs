use chrono::SecondsFormat;
use gluesql::{
    memory_storage::Key,
    prelude::{Glue, MemoryStorage, Payload},
};
use std::sync::{Arc, Mutex};

use crate::notification::Notification;
use crate::ArcGlue;

pub fn get_memory_glue() -> Glue<Key, MemoryStorage> {
    let storage = MemoryStorage::default();

    Glue::new(storage)
}

pub async fn initialize(glue: Arc<Mutex<Glue<Key, MemoryStorage>>>) {
    let mut glue = glue.lock().unwrap();

    let sqls = vec![
        "DROP TABLE IF EXISTS notifications;",
        r#"
        CREATE TABLE notifications (
            id INTEGER, description TEXT, 
            work_time INTEGER, break_time INTEGER, 
            created_at TIMESTAMP, 
            work_expired_at TIMESTAMP, break_expired_at TIMESTAMP,
        );"#,
    ];

    for sql in sqls {
        let output = glue.execute(sql).unwrap();
        debug!("output: {:?}", output);
    }
}

//TODO(young): Handle error
pub async fn create_notification(glue: ArcGlue, notification: &Notification) {
    let mut glue = glue.lock().unwrap();

    let (id, desc, work_time, break_time, created_at, w_expired_at, b_expired_at) =
        notification.get_values();

    let sql = format!(
        r#"
        INSERT INTO notifications VALUES ({}, '{}', {}, {}, '{}', '{}', '{}');
    "#,
        id,
        desc,
        work_time,
        break_time,
        created_at.to_rfc3339_opts(SecondsFormat::Millis, true),
        w_expired_at.to_rfc3339_opts(SecondsFormat::Millis, true),
        b_expired_at.to_rfc3339_opts(SecondsFormat::Millis, true)
    );

    debug!("create sql: {}", sql);

    let output = glue.execute(sql.as_str()).unwrap();
    debug!("output: {:?}", output);
}

pub async fn read_last_expired_notification(glue: ArcGlue) -> Option<Notification> {
    let mut glue = glue.lock().unwrap();

    let sql = String::from(
        r#"
        SELECT * FROM notifications ORDER BY break_expired_at DESC, work_expired_at DESC LIMIT 1;
        "#,
    );
    debug!("sql: {:?}", sql);

    let output = glue.execute(sql.as_str()).unwrap();
    debug!("output: {:?}", output);

    match output {
        Payload::Select {
            labels: _,
            mut rows,
        } => {
            if rows.is_empty() {
                None
            } else {
                Some(Notification::convert_to_notification(rows.swap_remove(0)))
            }
        }
        _ => {
            panic!("no such case!");
        }
    }
}

pub async fn read_notification(glue: ArcGlue, id: u16) -> Option<Notification> {
    let mut glue = glue.lock().unwrap();

    let sql = format!(
        r#"
        SELECT * FROM notifications WHERE id = {};
        "#,
        id
    );

    debug!("sql: {:?}", sql);

    let output = glue.execute(sql.as_str()).unwrap();
    debug!("output: {:?}", output);

    match output {
        Payload::Select {
            labels: _,
            mut rows,
        } => {
            if rows.is_empty() {
                None
            } else {
                Some(Notification::convert_to_notification(rows.swap_remove(0)))
            }
        }
        _ => {
            panic!("no such case!");
        }
    }
}

pub async fn list_notification(glue: ArcGlue) -> Vec<Notification> {
    let mut glue = glue.lock().unwrap();

    let sql = "SELECT * FROM notifications;";

    let output = glue.execute(sql).unwrap();
    debug!("output: {:?}", output);

    match output {
        Payload::Select { labels: _, rows } => rows
            .into_iter()
            .map(Notification::convert_to_notification)
            .collect(),
        _ => {
            panic!("no such case!");
        }
    }
}

pub async fn delete_notification(glue: ArcGlue, id: u16) {
    let mut glue = glue.lock().unwrap();

    // check if notification exists. It's okay. glue executes commands sequentially as of now.
    let sql = format!(
        r#"
        SELECT * FROM notifications WHERE id = {};
        "#,
        id
    );

    debug!("sql: {:?}", sql);

    let output = glue.execute(sql.as_str()).unwrap();
    debug!("output: {:?}", output);

    let sql = format!(
        r#"
        DELETE FROM notifications WHERE id = {};
        "#,
        id
    );

    debug!("delete sql: {}", sql);

    let output = glue.execute(sql.as_str()).unwrap();
    debug!("output: {:?}", output);
}

pub async fn delete_all_notification(glue: ArcGlue) {
    let mut glue = glue.lock().unwrap();

    let sql = r#"
        DELETE FROM notifications;
    "#;

    debug!("delete sql: {}", sql);

    let output = glue.execute(sql).unwrap();
    debug!("output: {:?}", output);
}

#[cfg(test)]
mod tests {
    use crate::notification::Notification;
    use chrono::Utc;
    use gluesql::prelude::{Payload, PayloadVariable};

    use super::{
        create_notification, delete_all_notification, delete_notification, get_memory_glue,
        initialize, list_notification, read_last_expired_notification, read_notification,
    };
    use std::{
        panic,
        sync::{Arc, Mutex},
    };

    #[tokio::test]
    async fn test_initialize_tables() {
        let glue = Arc::new(Mutex::new(get_memory_glue()));
        initialize(glue.clone()).await;

        let sql = "SHOW TABLES;";
        let output = glue.lock().unwrap().execute(sql).unwrap();

        match output {
            Payload::ShowVariable(PayloadVariable::Tables(names)) => {
                assert_eq!(1, names.len());
                assert_eq!("notifications", names[0]);
            }
            _ => {
                panic!("no such case");
            }
        }
    }

    #[tokio::test]
    async fn test_create_notification() {
        let glue = Arc::new(Mutex::new(get_memory_glue()));
        initialize(glue.clone()).await;

        let now = Utc::now();
        let notification = Notification::new(0, 25, 5, now);

        create_notification(glue.clone(), &notification).await;

        let result = read_notification(glue.clone(), 0).await.unwrap();
        assert_eq!(
            0,
            result.get_id(),
            "id are different {}, {}",
            0,
            result.get_id()
        );
        assert_eq!(
            now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            result
                .get_start_at()
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        );
    }

    #[tokio::test]
    async fn test_list_notification() {
        let glue = Arc::new(Mutex::new(get_memory_glue()));
        initialize(glue.clone()).await;

        // empty row
        let result = list_notification(glue.clone()).await;
        assert_eq!(0, result.len());

        let now = Utc::now();
        let notification = Notification::new(0, 25, 5, now);
        create_notification(glue.clone(), &notification).await;

        let notification = Notification::new(1, 30, 10, now);
        create_notification(glue.clone(), &notification).await;

        let result = list_notification(glue.clone()).await;
        assert_eq!(2, result.len());
        assert_eq!(0, result[0].get_id());
        assert_eq!(1, result[1].get_id());
    }

    #[tokio::test]
    async fn test_delete_notification() {
        let glue = Arc::new(Mutex::new(get_memory_glue()));
        initialize(glue.clone()).await;

        let now = Utc::now();
        let notification = Notification::new(0, 25, 5, now);
        create_notification(glue.clone(), &notification).await;

        delete_notification(glue.clone(), 0).await;
        let result = read_notification(glue.clone(), 0).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_all_notifications() {
        let glue = Arc::new(Mutex::new(get_memory_glue()));
        initialize(glue.clone()).await;

        let now = Utc::now();
        let notification = Notification::new(0, 25, 5, now);
        create_notification(glue.clone(), &notification).await;

        let notification = Notification::new(1, 30, 10, now);
        create_notification(glue.clone(), &notification).await;

        delete_all_notification(glue.clone()).await;
        let result = list_notification(glue.clone()).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_read_last_expired_notification() {
        let glue = Arc::new(Mutex::new(get_memory_glue()));
        initialize(glue.clone()).await;

        let now = Utc::now();
        let notification = Notification::new(0, 25, 5, now);
        create_notification(glue.clone(), &notification).await;

        let notification = Notification::new(1, 30, 10, now);
        create_notification(glue.clone(), &notification).await;

        let result = read_last_expired_notification(glue.clone()).await;
        assert!(result.is_some());
        assert_eq!(1, result.unwrap().get_id());
    }
}
