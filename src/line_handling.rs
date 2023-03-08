use crate::{InputSource, UserInput};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

/// Handle all cli input events with rustyline
pub fn line_handler(tx: Sender<UserInput>) -> JoinHandle<()> {
    let mut rl = DefaultEditor::new().expect("Could not initiate editor");

    // load history if available. Will create a new file if it didn't exist at saving time.
    // ? change file location to somewhere discreet?
    if rl.load_history("history.txt").is_err() {
        debug!("No previous history.");
    }

    // essentially tries to copy what `spawn_stdinput_handler` did with stdin.read_line
    // let rustyline handle the input events instead. Keep the same architecture as before
    tokio::spawn(async move {
        loop {
            // set up what to show at the beginning of the line
            let readline = rl.readline("> ");

            match readline {
                Ok(line) => {
                    // add each line to history so arrow up/down key can work
                    rl.add_history_entry(line.as_str()).unwrap();

                    // exit command does std::process::exit(0). So save whatever we got this session
                    if line.trim() == "exit" {
                        rl.save_history("history.txt").unwrap();
                    }

                    let _ = tx
                        .send(UserInput {
                            input: line,
                            source: InputSource::StandardInput,
                        })
                        .await;
                }
                // handles the CTRL + C event
                Err(ReadlineError::Interrupted) => {
                    rl.save_history("history.txt").unwrap();
                    std::process::exit(0);
                }
                Err(ReadlineError::Eof) => {
                    rl.save_history("history.txt").unwrap();
                    std::process::exit(0);
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    })
}
