use std::io::{self, BufRead, Write};

pub fn read_command<R, W>(stdout: &mut W, stdin: &mut R) -> String
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

#[cfg(test)]
mod tests {
    use super::read_command;

    #[test]
    fn test_read_command() {
        let mut input = &b"list"[..];
        let mut output = Vec::new();

        let command = read_command(&mut output, &mut input);
        assert_eq!("list", command);
    }
}
