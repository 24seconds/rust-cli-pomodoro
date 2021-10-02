use std::io::{self, Write};

#[tokio::main]
async fn main() {
    loop {
        println!("try to read command");
        let command = read_command();
        println!("user input: {}", command)
    }
}

fn read_command() -> String {
    print!("> ");

    // ignore error
    let _ = io::stdout().flush().unwrap();

    let mut command = String::new();

    io::stdin()
        .read_line(&mut command)
        .expect("Failed to read line");

    let command = command.trim().to_string();

    command
}
