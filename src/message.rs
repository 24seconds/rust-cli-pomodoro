pub enum Message {
    UserInput { command: String },
    Create {
        id: u16,
        work_time: u16,
        break_time: u16,
    },
    Delete {
        id: u16,
    },
    DeleteAll,
    Query,
}
