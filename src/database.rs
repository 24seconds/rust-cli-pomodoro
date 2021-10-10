use chrono::SecondsFormat;
use gluesql::{memory_storage::Key, Glue, MemoryStorage, Payload};

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
            PRIMARY KEY(id)
        );"#,
        r#"
        INSERT INTO notifications VALUES (
            1, 'test-notification-1', '2021-10-01T19:41:42Z', 
            '2021-10-01T19:42:42Z', '2021-10-01T19:45:42Z'
        );"#,
        r#"
        INSERT INTO notifications VALUES (
            2, 'test-notification-2', '2021-10-01T19:41:42Z', 
            '2021-10-01T19:43:42Z', '2021-10-01T19:47:42Z'
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

    println!("create sql: {}", sql);

    let output = glue.execute_async(sql.as_str()).await.unwrap();
    println!("output: {:?}", output);
}

pub async fn list_notification(glue: &mut Glue<Key, MemoryStorage>) {
    let sql = "SELECT * FROM notifications;";

    let output = glue.execute_async(sql).await.unwrap();
    println!("output: {:?}", output);

    match output {
        Payload::Select { labels: _, rows } => {
            for row in rows.into_iter() {
                println!("ROW: {:?}", row);
            }
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

    println!("delete sql: {}", sql);

    let output = glue.execute_async(sql.as_str()).await.unwrap();
    println!("output: {:?}", output);
}

pub async fn delete_all_notification(glue: &mut Glue<Key, MemoryStorage>) {
    let sql = r#"
        DELETE FROM notifications;
    "#;

    println!("delete sql: {}", sql);

    let output = glue.execute_async(sql).await.unwrap();
    println!("output: {:?}", output);
}
