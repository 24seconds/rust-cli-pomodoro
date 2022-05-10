use std::io::{BufRead, Write};
use std::{error::Error, str::FromStr};

use clap::ArgMatches;

use crate::command::{DEFAULT_BREAK_TIME, DEFAULT_WORK_TIME};

pub fn parse_work_and_break_time(matches: &ArgMatches) -> Result<(u16, u16), Box<dyn Error>> {
    let (work_time, break_time) = if matches.is_present("default") {
        (DEFAULT_WORK_TIME, DEFAULT_BREAK_TIME)
    } else {
        let work_time = parse_arg::<u16>(matches, "work")?;
        let break_time = parse_arg::<u16>(matches, "break")?;

        (work_time, break_time)
    };

    Ok((work_time, break_time))
}

pub fn parse_arg<C>(arg_matches: &ArgMatches, arg_name: &str) -> Result<C, Box<dyn Error>>
where
    C: FromStr,
{
    let str = arg_matches
        .value_of(arg_name)
        .ok_or(format!("failed to get ({}) from cli", arg_name))?;

    let parsed = str
        .parse::<C>()
        .map_err(|_| format!("failed to parse arg ({})", str))?;

    Ok(parsed)
}

pub fn read_input<R, W>(stdout: &mut W, stdin: &mut R) -> String
where
    R: BufRead,
    W: Write,
{
    write!(stdout, "> ").unwrap();
    stdout.flush().expect("could not flush stdout");

    let mut command = String::new();

    stdin.read_line(&mut command).expect("failed to read line");
    let command = command.trim().to_string();

    command
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

    fn test_parse_arg() {
        let m = Command::new("myapp")
            .arg(Arg::new("id").takes_value(true))
            .get_matches_from("myapp abc".split_whitespace());

        // parse as expected
        let id = parse_arg::<String>(&m, "id").unwrap_or_else(|e| panic!("An error occurs: {}", e));
        assert!(id.eq("abc"));

        let m = Command::new("myapp")
            .arg(Arg::new("id").takes_value(true))
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