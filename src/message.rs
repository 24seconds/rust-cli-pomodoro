pub enum Message {
    UserInput { command: String },
    Delete { id: u16 },
}
