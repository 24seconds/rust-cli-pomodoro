fn get_binary_name() -> String {
    let binary_name = env!("CARGO_BIN_NAME");
    binary_name.to_string()
}

pub fn initialize_logging() {
    let package_name = &get_binary_name();

    if cfg!(debug_assertions) {
        env_logger::Builder::from_default_env()
            .filter(Some(package_name), log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter(Some(package_name), log::LevelFilter::Info)
            .init();
    }
}

#[cfg(test)]
mod tests {
    use super::get_binary_name;

    #[test]
    fn test_get_binary_name() {
        let binary_name = get_binary_name();
        assert_eq!("pomodoro", binary_name);
    }
}
