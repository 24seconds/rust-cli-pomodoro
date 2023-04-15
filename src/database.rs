use chrono::SecondsFormat;
use gluesql::core::ast_builder::{table, Build};
use gluesql::prelude::{Glue, MemoryStorage, Payload};
use std::sync::{Arc, Mutex};

use crate::ArcGlue;
use crate::{archived_notification::ArchivedNotification, notification::Notification};

pub fn get_memory_glue() -> Glue<MemoryStorage> {
    let storage = MemoryStorage::default();

    Glue::new(storage)
}

pub async fn initialize(glue: Arc<Mutex<Glue<MemoryStorage>>>) {
    let mut glue = glue.lock().unwrap();

    let sql_stmts = vec![
        table("notifications")
            .drop_table_if_exists()
            .build()
            .unwrap(),
        table("notifications")
            .create_table()
            .add_column("id INTEGER")
            .add_column("description TEXT")
            .add_column("work_time INTEGER")
            .add_column("break_time INTEGER")
            .add_column("created_at TIMESTAMP")
            .add_column("work_expired_at TIMESTAMP")
            .add_column("break_expired_at TIMESTAMP")
            .build()
            .unwrap(),
        table("archived_notifications")
            .drop_table_if_exists()
            .build()
            .unwrap(),
        table("archived_notifications")
            .create_table()
            .add_column("id INTEGER")
            .add_column("description TEXT")
            .add_column("work_time INTEGER")
            .add_column("break_time INTEGER")
            .add_column("created_at TIMESTAMP")
            .add_column("work_expired_at TIMESTAMP")
            .add_column("break_expired_at TIMESTAMP")
            .build()
            .unwrap(),
    ];

    for stmt in sql_stmts {
        let output = glue.execute_stmt(&stmt).unwrap();
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

    let output = glue.execute(sql.as_str()).unwrap().swap_remove(0);
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

    let sql_stmt = table("notifications")
        .select()
        .filter(format!("id = {}", id).as_str())
        .build()
        .unwrap();
    debug!("sql_stmt: {:?}", sql_stmt);

    let output = glue.execute_stmt(&sql_stmt).unwrap();
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

    let sql_stmt = table("notifications").select().build().unwrap();

    let output = glue.execute_stmt(&sql_stmt).unwrap();
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

pub async fn list_archived_notification(glue: ArcGlue) -> Vec<ArchivedNotification> {
    let mut glue = glue.lock().unwrap();

    let sql = "SELECT * FROM archived_notifications ORDER BY id DESC;";

    let output = glue.execute(sql).unwrap().swap_remove(0);
    debug!("output: {:?}", output);

    match output {
        Payload::Select { labels: _, rows } => rows
            .into_iter()
            // TODO(young): As of now archived_notifications schema is same as notifications table
            .map(Notification::convert_to_notification)
            .map(ArchivedNotification::from)
            .collect(),
        _ => {
            panic!("no such case!");
        }
    }
}

pub async fn delete_and_archive_notification(glue: ArcGlue, id: u16) {
    archive_notification(glue.clone(), id).await;
    delete_notification(glue.clone(), id).await;
}

//TODO(young): Handle error?
pub async fn delete_notification(glue: ArcGlue, id: u16) {
    let mut glue = glue.lock().unwrap();

    // check if notification exists. It's okay. glue executes commands sequentially as of now.
    let sql_stmt = table("notifications")
        .select()
        .filter(format!("id = {}", id).as_str())
        .build()
        .unwrap();
    debug!("sql_stmt: {:?}", sql_stmt);

    let output = glue.execute_stmt(&sql_stmt).unwrap();
    debug!("output: {:?}", output);

    let sql_stmt = table("notifications")
        .delete()
        .filter(format!("id = {}", id).as_str())
        .build()
        .unwrap();
    debug!("delete sql_stmt: {:?}", sql_stmt);

    let output = glue.execute_stmt(&sql_stmt).unwrap();
    debug!("output: {:?}", output);
}

//TODO(young): Handle error?
pub async fn archive_notification(glue: ArcGlue, id: u16) {
    let mut glue = glue.lock().unwrap();

    let sql = format!(
        r#"
    INSERT INTO archived_notifications
    SELECT * FROM notifications WHERE id = {};
    "#,
        id
    );

    debug!("sql: {:?}", sql);

    let output = glue.execute(sql.as_str()).unwrap();
    debug!("output: {:?}", output);
}

pub async fn archive_all_notification(glue: ArcGlue) {
    let mut glue = glue.lock().unwrap();

    let sql = r#"
    INSERT INTO archived_notifications
    SELECT * FROM notifications;
    "#;

    debug!("archive all sql: {}", sql);

    let output = glue.execute(sql).unwrap();
    debug!("output: {:?}", output);
}

pub async fn delete_all_archived_notification(glue: ArcGlue) {
    let mut glue = glue.lock().unwrap();

    let sql = r#"
    DELETE FROM archived_notifications;
    "#;

    debug!("delete all sql: {}", sql);

    let output = glue.execute(sql).unwrap();
    debug!("output: {:?}", output);
}

pub async fn delete_all_notification(glue: ArcGlue) {
    let mut glue = glue.lock().unwrap();

    let sql_stmt = table("notifications").delete().build().unwrap();
    debug!("delete sql_stmt: {:?}", sql_stmt);

    let output = glue.execute_stmt(&sql_stmt).unwrap();
    debug!("output: {:?}", output);
}

pub async fn delete_and_archive_all_notification(glue: ArcGlue) {
    archive_all_notification(glue.clone()).await;
    delete_all_notification(glue.clone()).await;
}

#[cfg(test)]
mod tests {
    use crate::notification::Notification;
    use chrono::Utc;
    use gluesql::prelude::{Payload, PayloadVariable};

    use super::{
        archive_all_notification, archive_notification, create_notification,
        delete_all_archived_notification, delete_all_notification, delete_notification,
        get_memory_glue, initialize, list_archived_notification, list_notification,
        read_last_expired_notification, read_notification,
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
        let output = glue.lock().unwrap().execute(sql).unwrap().swap_remove(0);

        match output {
            Payload::ShowVariable(PayloadVariable::Tables(mut names)) => {
                names.sort();

                assert_eq!(2, names.len());
                assert_eq!("archived_notifications", names[0]);
                assert_eq!("notifications", names[1]);
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
    async fn test_archive_notification() {
        let glue = Arc::new(Mutex::new(get_memory_glue()));
        initialize(glue.clone()).await;

        let now = Utc::now();
        let notification = Notification::new(0, 25, 5, now);
        create_notification(glue.clone(), &notification).await;

        let notification = Notification::new(1, 30, 10, now);
        create_notification(glue.clone(), &notification).await;

        archive_all_notification(glue.clone()).await;

        let result = list_notification(glue.clone()).await;
        assert!(result.len() == 2);

        let result = list_archived_notification(glue.clone()).await;
        assert!(result.len() == 2);
    }

    #[tokio::test]
    async fn test_archive_all_notification() {
        let glue = Arc::new(Mutex::new(get_memory_glue()));
        initialize(glue.clone()).await;

        let now = Utc::now();
        let notification = Notification::new(0, 25, 5, now);
        create_notification(glue.clone(), &notification).await;

        archive_notification(glue.clone(), 0).await;

        let result = list_notification(glue.clone()).await;
        assert!(result.len() == 1);

        let result = list_archived_notification(glue.clone()).await;
        assert!(result.len() == 1);
    }

    #[tokio::test]
    async fn test_delete_all_archived_notification() {
        let glue = Arc::new(Mutex::new(get_memory_glue()));
        initialize(glue.clone()).await;

        let now = Utc::now();
        let notification = Notification::new(0, 25, 5, now);
        create_notification(glue.clone(), &notification).await;

        archive_notification(glue.clone(), 0).await;

        let result = list_archived_notification(glue.clone()).await;
        assert!(result.len() == 1);

        delete_all_archived_notification(glue.clone()).await;

        let result = list_archived_notification(glue.clone()).await;
        assert!(result.len() == 0);
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
