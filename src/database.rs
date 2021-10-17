use chrono::SecondsFormat;
use gluesql::{memory_storage::Key, Glue, MemoryStorage, Payload};
use tabled::Table;

use crate::notification::Notification;

pub fn get_memory_glue() -> Glue<Key, MemoryStorage> {
    let storage = MemoryStorage::default();

    Glue::new(storage)
}

pub async fn initialize(glue: &mut Glue<Key, MemoryStorage>) {
    let sqls = vec![
        "DROP TABLE IF EXISTS notifications;",
        r#"
        CREATE TABLE notifications (
            id INTEGER, description TEXT, 
            created_at TIMESTAMP, 
            work_expired_at TIMESTAMP, break_expired_at TIMESTAMP,
        );"#,
    ];

    for sql in sqls {
        let output = glue.execute_async(sql).await.unwrap();
        println!("output: {:?}", output);
    }
}

//TODO(young): Handle error
pub async fn create_notification(glue: &mut Glue<Key, MemoryStorage>, notification: &Notification) {
    let (id, desc, created_at, w_expired_at, b_expired_at) = notification.get_values();

    let sql = format!(
        r#"
        INSERT INTO notifications VALUES ({}, '{}', '{}', '{}', '{}');
    "#,
        id,
        desc,
        created_at.to_rfc3339_opts(SecondsFormat::Millis, true),
        w_expired_at.to_rfc3339_opts(SecondsFormat::Millis, true),
        b_expired_at.to_rfc3339_opts(SecondsFormat::Millis, true)
    );

    debug!("create sql: {}", sql);

    let output = glue.execute_async(sql.as_str()).await.unwrap();
    debug!("output: {:?}", output);
}

pub async fn list_notification(glue: &mut Glue<Key, MemoryStorage>) {
    let sql = "SELECT * FROM notifications;";

    let output = glue.execute_async(sql).await.unwrap();
    debug!("output: {:?}", output);

    match output {
        Payload::Select { labels: _, rows } => {
            let notifications = rows.into_iter().map(Notification::convert_to_notification);

            let table = Table::new(notifications).to_string();

            info!("\n{}", table);
        }
        _ => {
            panic!("no such case!");
        }
    }
}

pub async fn delete_notification(glue: &mut Glue<Key, MemoryStorage>, id: u16) {
    let sql = format!(
        r#"
        DELETE FROM notifications WHERE id = {};
        "#,
        id
    );

    debug!("delete sql: {}", sql);

    let output = glue.execute_async(sql.as_str()).await.unwrap();
    debug!("output: {:?}", output);
}

pub async fn delete_all_notification(glue: &mut Glue<Key, MemoryStorage>) {
    let sql = r#"
        DELETE FROM notifications;
    "#;

    debug!("delete sql: {}", sql);

    let output = glue.execute_async(sql).await.unwrap();
    debug!("output: {:?}", output);
}
