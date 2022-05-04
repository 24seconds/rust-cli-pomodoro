use crate::argument::{parse_arg, DEFAULT_BREAK_TIME, DEFAULT_WORK_TIME};
use crate::Notification;
use chrono::{DateTime, Utc};
use clap::ArgMatches;
use clap_v3;
use std::error::Error;
use std::io::{BufRead, Write};

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

pub fn get_new_notification(
    matches: &clap_v3::ArgMatches,
    id_manager: &mut u16,
    created_at: DateTime<Utc>,
) -> Result<Option<Notification>, Box<dyn Error>> {
    let (work_time, break_time) = if matches.is_present("default") {
        (DEFAULT_WORK_TIME, DEFAULT_BREAK_TIME)
    } else {
        let work_time = parse_arg::<u16>(matches, "work")?;
        let break_time = parse_arg::<u16>(matches, "break")?;

        (work_time, break_time)
    };

    debug!("work_time: {}", work_time);
    debug!("break_time: {}", break_time);

    if work_time == 0 && break_time == 0 {
        eprintln!("work_time and break_time both can not be zero both");
        // TODO: This shouldn't return Ok, since it is an error, but for now,
        // is just a "temporal fix" for returning from the function.
        return Ok(None);
    }

    let id = get_new_id(id_manager);

    Ok(Some(Notification::new(
        id, work_time, break_time, created_at,
    )))
}

fn get_new_id(id_manager: &mut u16) -> u16 {
    let id = *id_manager;
    *id_manager += 1;

    id
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use clap::App;

    use crate::argument::add_args_for_creation;

    use super::{get_new_notification, read_input};

    #[test]
    fn test_read_command() {
        let mut input = &b"list"[..];
        let mut output = Vec::new();

        let command = read_input(&mut output, &mut input);
        assert_eq!("list", command);
    }

    #[test]
    fn test_get_new_notification() {
        let app = App::new("myapp");
        let matches =
            add_args_for_creation(app).get_matches_from("myapp -w 25 -b 5".split_whitespace());
        let mut id_manager = 0;
        let now = Utc::now();

        let notification = get_new_notification(&matches, &mut id_manager, now).unwrap();
        assert!(notification.is_some());
        let notification = notification.unwrap();

        let (id, _, wt, bt, created_at, _, _) = notification.get_values();
        assert_eq!(0, id);
        assert_eq!(25, wt);
        assert_eq!(5, bt);
        assert_eq!(now, created_at);
    }
}
