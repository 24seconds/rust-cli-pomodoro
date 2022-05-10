pub enum OutputType {
    Info,
    Error,
    Print,
    Println,
}

pub struct OutputAccumulater {
    body: Vec<String>,
}

impl OutputAccumulater {
    pub fn new() -> Self {
        OutputAccumulater { body: Vec::new() }
    }

    pub fn push(&mut self, r#type: OutputType, message: String) {
        match r#type {
            OutputType::Info => {
                info!("{}", message);
            }
            OutputType::Error => {
                eprintln!("{}", message);
            }
            OutputType::Print => {
                print!("{}", message);
            }
            OutputType::Println => {
                println!("{}", message);
            }
        }

        self.body.push(message);
    }

    // take_body extract messages. After call, OutputAccumulater has empty body
    pub fn take_body(&mut self) -> Vec<String> {
        std::mem::take(&mut self.body)
    }
}
