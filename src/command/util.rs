use clap::ArgMatches;
use clap_complete::Shell;
use std::io::{self, Write};
use std::str::FromStr;
use std::sync::Arc;

use crate::command::{DEFAULT_BREAK_TIME, DEFAULT_WORK_TIME};
use crate::configuration::Configuration;
use crate::error::ParseError;

pub fn parse_work_and_break_time(
    matches: &ArgMatches,
    configuration: Option<&Arc<Configuration>>,
) -> Result<(Option<u16>, Option<u16>), ParseError> {
    if let Some(conf) = configuration {
        let mut work_time = match conf.get_work_time() {
            Some(work_time) => work_time,
            None => DEFAULT_WORK_TIME,
        };

        let mut break_time = match conf.get_break_time() {
            Some(break_time) => break_time,
            None => DEFAULT_BREAK_TIME,
        };

        if let Ok(val) = parse_arg::<u16>(matches, "work") {
            work_time = val;
        };

        if let Ok(val) = parse_arg::<u16>(matches, "break") {
            break_time = val;
        };

        Ok((Some(work_time), Some(break_time)))
    } else {
        let mut work_time = None;
        let mut break_time = None;

        if let Ok(val) = parse_arg::<u16>(matches, "work") {
            work_time = Some(val);
        }

        if let Ok(val) = parse_arg::<u16>(matches, "break") {
            break_time = Some(val);
        }
        Ok((work_time, break_time))
    }
}

pub fn parse_shell(matches: &ArgMatches) -> Option<Shell> {
    let shell = matches.get_one::<String>("shell");
    if let Some(shell) = shell {
        match shell.as_str() {
            "fish" => Some(Shell::Fish),
            "zsh" => Some(Shell::Zsh),
            "bash" => Some(Shell::Bash),
            "elvish" => Some(Shell::Elvish),
            "powershell" => Some(Shell::PowerShell),
            _ => None,
        }
    } else {
        None
    }
}

pub fn parse_arg<C>(arg_matches: &ArgMatches, arg_name: &str) -> Result<C, ParseError>
where
    C: FromStr,
{
    let str = arg_matches
        .get_one::<String>(arg_name)
        .ok_or(format!("failed to get ({}) from cli", arg_name))
        .map_err(ParseError::new)?;

    let parsed = str
        .parse::<C>()
        .map_err(|_| format!("failed to parse arg ({})", str))
        .map_err(ParseError::new)?;

    Ok(parsed)
}

pub fn print_start_up() {
    let stdout = &mut io::stdout();
    write!(stdout, "> ").unwrap();
    stdout.flush().expect("could not flush stdout");
}

pub fn write_output<W>(stdout: &mut W)
where
    W: Write,
{
    write!(stdout, "> ").unwrap();
    stdout.flush().expect("couldn't flush stdout");
}

#[cfg(test)]
mod tests {
    use clap::{Arg, Command};

    use super::parse_arg;

    #[test]
    fn test_parse_arg() {
        let m = Command::new("myapp")
            .arg(Arg::new("id").num_args(1))
            .get_matches_from("myapp abc".split_whitespace());

        // parse as expected
        let id = parse_arg::<String>(&m, "id").unwrap_or_else(|e| panic!("An error occurs: {}", e));
        assert!(id.eq("abc"));

        let m = Command::new("myapp")
            .arg(Arg::new("id").num_args(1))
            .get_matches_from("myapp abc".split_whitespace());

        // error when parsing
        let id = parse_arg::<u16>(&m, "id");
        assert!(id.is_err());
        assert!(id
            .err()
            .unwrap()
            .to_string()
            .contains("failed to parse arg"));
    }
}
