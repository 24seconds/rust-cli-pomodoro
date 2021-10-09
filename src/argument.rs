use clap::{App, AppSettings, Arg, SubCommand};


pub fn get_app() -> App<'static, 'static> {
    App::new("pomodoro")
        .setting(AppSettings::NoBinaryName)
        .version("0.0.1")
        .author("Young")
        .about("manage your time!")
        .subcommands(vec![
            SubCommand::with_name("create")
                .about("create a notification")
                .arg(
                    Arg::with_name("work")
                        .help("The focus time. Unit is minutes")
                        .takes_value(true)
                        .short("w")
                        .long("work"),
                )
                .arg(
                    Arg::with_name("break")
                        .help("The break time, Unit is minutes")
                        .takes_value(true)
                        .short("b")
                        .long("b"),
                ), // TODO(young): add default argument.
                   // TODO(young): Check is possible to detect
                   // TODO(young): if default arg is specified then other args should not be specified.
        ])
}
