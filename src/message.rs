use tokio::sync::oneshot;

pub enum Message {
    UserInput {
        command: String,
        oneshot_tx: oneshot::Sender<bool>,
    },
    Create {
        id: u16,
        work_time: u16,
        break_time: u16,
        oneshot_tx: oneshot::Sender<bool>,
    },
    Delete {
        id: u16,
        oneshot_tx: oneshot::Sender<bool>,
    },
    SilentDelete {
        id: u16,
    },
    DeleteAll {
        oneshot_tx: oneshot::Sender<bool>,
    },
    Query {
        oneshot_tx: oneshot::Sender<bool>,
    },
    NotificationTest {
        oneshot_tx: oneshot::Sender<bool>,
    }
}
