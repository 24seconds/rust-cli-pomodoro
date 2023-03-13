use crate::{InputSource, UserInput};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::process;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

/// Handles all cli input events with rustyline
pub fn handle(tx: Sender<UserInput>) -> JoinHandle<()> {
    let mut rl = DefaultEditor::new().unwrap_or_else(|err| {
        println!(
            "Something went wrong. Could not initiate editor. Error: {}",
            err
        );
        process::exit(1);
    });

    tokio::spawn(async move {
        loop {
            // set up what to show at the beginning of the line
            let readline = rl.readline("> ");

            match readline {
                Ok(line) => {
                    // add each line to history so arrow up/down key can work
                    rl.add_history_entry(line.as_str()).unwrap();

                    let _ = tx
                        .send(UserInput {
                            input: line,
                            source: InputSource::StandardInput,
                        })
                        .await;
                }
                // handles the CTRL + C event
                Err(ReadlineError::Interrupted) => {
                    process::exit(0);
                }
                Err(ReadlineError::Eof) => {
                    process::exit(0);
                }
                Err(err) => {
                    println!("Something went wrong. Error: {:?}", err);
                    process::exit(0);
                }
            }
        }
    })
}
