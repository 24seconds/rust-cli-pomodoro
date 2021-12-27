use chrono::SecondsFormat;
use gluesql::{
    prelude::{Glue, MemoryStorage, Payload},
    storages::memory_storage::Key,
};
use std::sync::{Arc, Mutex};
use tabled::{Style, Table};

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

pub async fn list_notification(glue: ArcGlue) {
    let mut glue = glue.lock().unwrap();

    let sql = "SELECT * FROM notifications;";

    let output = glue.execute(sql).unwrap();
    debug!("output: {:?}", output);

    match output {
        Payload::Select { labels: _, rows } => {
            let notifications = rows.into_iter().map(Notification::convert_to_notification);

            let table = Table::new(notifications)
                .with(Style::pseudo_clean())
                .to_string();

            info!("\n{}", table);
        }
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
