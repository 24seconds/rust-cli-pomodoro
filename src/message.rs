use tokio::sync::oneshot;

pub enum Message {
    UserInput {
        command: String,
        oneshot_tx: oneshot::Sender<String>,
    },
    Create {
        id: u16,
        work_time: u16,
        break_time: u16,
        oneshot_tx: oneshot::Sender<String>,
    },
    Delete {
        id: u16,
        oneshot_tx: oneshot::Sender<String>,
    },
    SilentDelete {
        id: u16,
    },
    DeleteAll {
        oneshot_tx: oneshot::Sender<String>,
    },
    Query {
        oneshot_tx: oneshot::Sender<String>,
    },
    NotificationTest {
        oneshot_tx: oneshot::Sender<String>,
    },
}
