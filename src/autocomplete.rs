use clap::Command;
use clap_complete::{generate, Generator, Shell};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::{Command as cmd, Stdio};

use crate::command::application::get_main_command;

/// Generate auto completion file for a given shell and write it to the file
pub fn generate_completion<G: Generator>(
    gen: G,
    cmd: &mut Command,
    file_name: &str,
) -> std::io::Result<()> {
    let mut file = File::create(file_name)?;
    generate(gen, cmd, cmd.get_name().to_string(), &mut file);
    Ok(())
}

/// Adds auto completion for the shell that is being used
pub fn add_autocomplete() -> Result<(), Box<dyn std::error::Error>> {
    let current_shell = get_current_shell();

    if let Ok(shell) = current_shell {
        debug!("Current shell: {}", shell);
        let mut main_command = get_main_command();
        let (file_name, mut moving_path) = match shell {
            Shell::Fish => ("pomodoro.fish", validate_path(Shell::Fish)?),
            Shell::Zsh => ("_pomodoro.zsh", validate_path(Shell::Zsh)?),
            //Shell::Elvish => ("pomodoro.elv", validate_path(Shell::Elvish)?),
            Shell::Bash => ("pomodoro.bash", validate_path(Shell::Bash)?),
            _ => return Err("Invalid Shell".into()),
        };

        moving_path.push(file_name);
        debug!("Autocomplete file location: {:?}", moving_path);

        // do not generate any file if already exists
        if verify_autocomplete(&moving_path) {
            debug!("Autocomplete file already exists. Stopping procedure");
            return Ok(());
        }

        generate_completion(shell, &mut main_command, file_name)?;

        // move the generated file -> remove -> Edit the shell file if required
        fs::copy(file_name, moving_path)?;
        fs::remove_file(file_name)?;
        edit_shell_file(shell)?;
    }
    Ok(())
}

/// Returns the current shell
fn get_current_shell() -> Result<Shell, Box<dyn std::error::Error>> {
    if cfg!(target_os = "linux") {
        // get the parent process id from where we can get the current shell
        let parent_process = cmd::new("ps")
            .arg("-p")
            .arg(std::process::id().to_string())
            .arg("-o")
            .arg("ppid=")
            .stdout(Stdio::piped())
            .output()?;

        // convert to a usable id
        let stdout = String::from_utf8(parent_process.stdout)?;
        let ppid = stdout.trim().parse::<i32>()?;

        // Use PS command again to get the shell
        let output = cmd::new("ps")
            .arg("-p")
            .arg(ppid.to_string())
            .arg("-o")
            .arg("comm=")
            .stdout(Stdio::piped())
            .output()?;

        // convert to readable version
        let stdout = String::from_utf8(output.stdout)?;
        let shell = stdout.trim();

        match shell {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            "fish" => Ok(Shell::Fish),
            //"elvish" => Ok(Shell::Elvish),
            _ => return Err("Unknown Shell Found".into()),
        }
    } else {
        Err("Could not find the shell".into())
    }
}

/// Checks if the autocomplete file already exists
fn verify_autocomplete(location: &PathBuf) -> bool {
    match fs::metadata(location) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Verifies if shell path already exists or creates them
fn validate_path(shell: Shell) -> Result<PathBuf, std::io::Error> {
    let home_dir = match std::env::var("HOME") {
        Ok(val) => val,
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "HOME environment variable not set",
            ))
        }
    };
    let mut function_dir = PathBuf::new();
    match shell {
        // * except restarting the shell, no other step is required
        Shell::Fish => function_dir.push(format!("{}/.config/fish/functions", home_dir)),

        // * 'autoload -U compinit && compinit' this will reload completion data for the completion to work for pomodoro
        // * restarting pc may also work or properly restarting the shell
        Shell::Zsh => function_dir.push(format!("{}/.zsh/completion", home_dir)),
        //Shell::Elvish => function_dir.push(format!("{}/.elvish/completers/", home_dir)),

        // * restarting the shell is enough
        Shell::Bash => function_dir.push(format!("{}/.bash_completion", home_dir)),
        _ => {}
    };

    // create all the path if it doesn't exist
    if let Err(err) = fs::metadata(&function_dir) {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                fs::create_dir_all(&function_dir)?;
            }
            _ => return Err(err),
        }
    }

    Ok(function_dir)
}

/// Edits shell file for shells that requires extra steps to load the auto completion file
fn edit_shell_file(shell: Shell) -> Result<(), std::io::Error> {
    // get the home dir
    let home_dir = match std::env::var("HOME") {
        Ok(val) => val,
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "HOME environment variable not set",
            ))
        }
    };

    // store the path where the file needs to be edited
    // and the exact part that needs to added in the file
    let mut file_path = String::new();
    let mut to_add = String::new();

    match shell {
        Shell::Zsh => {
            file_path = format!("{}/.zshrc", home_dir);
            to_add = "fpath+=(~/.zsh/completion)
autoload -U compinit && compinit"
                .to_string();
        }
        Shell::Bash => {
            file_path = format!("{}/.bashrc", home_dir);
            to_add = "source ~/.bash_completion/pomodoro.bash".to_string();
        }
        _ => {}
    }

    if !file_path.is_empty() && !to_add.is_empty() {
        // Open the file for reading
        let file = File::open(&file_path)?;
        let reader = BufReader::new(file);

        // Read the contents of the file into a vector of strings
        let mut lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

        // Check if the line already exists in the file
        if !lines.contains(&to_add.to_string()) {
            debug!("Adding to {}: {}", file_path, to_add);
            // Append the line to the end of the vector if it doesn't exist
            lines.push(to_add.to_string());
        }

        // Open the file for writing and write the modified contents to it
        let file = File::create(&file_path)?;
        let mut writer = BufWriter::new(file);
        for line in lines {
            writeln!(writer, "{}", line)?;
        }
    }
    Ok(())
}
