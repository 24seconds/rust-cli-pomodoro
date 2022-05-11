use crate::{
    command::application::{CLEAR, CREATE, DELETE, EXIT, HISTORY, LIST, LS, Q, QUEUE, TEST},
    error::ParseError,
};

pub enum ActionType {
    Create,
    Queue,
    Delete,
    List,
    Test,
    Exit,
    Clear,
    History,
}

impl ActionType {
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        match s.to_lowercase().as_str() {
            CREATE => Ok(ActionType::Create),
            Q | QUEUE => Ok(ActionType::Queue),
            DELETE => Ok(ActionType::Delete),
            LS | LIST => Ok(ActionType::List),
            TEST => Ok(ActionType::Test),
            EXIT => Ok(ActionType::Exit),
            CLEAR => Ok(ActionType::Clear),
            HISTORY => Ok(ActionType::History),
            _ => Err(ParseError::new(format!(
                "failed to parse str ({}) to ActionType",
                s
            ))),
        }
    }
}

impl From<ActionType> for String {
    fn from(action: ActionType) -> Self {
        match action {
            ActionType::Create => String::from(CREATE),
            ActionType::Queue => String::from(QUEUE),
            ActionType::Delete => String::from(DELETE),
            ActionType::List => String::from(LIST),
            ActionType::Test => String::from(TEST),
            ActionType::Exit => String::from(EXIT),
            ActionType::Clear => String::from(CLEAR),
            ActionType::History => String::from(HISTORY),
        }
    }
}
