fn get_package_name() -> String {
    let package_name = env!("CARGO_PKG_NAME");
    package_name.replace('-', "_")
}

pub fn initialize_logging() {
    let package_name = &get_package_name();

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
    use super::get_package_name;

    #[test]
    fn test_get_package_name() {
        let package_name = get_package_name();
        assert_eq!("rust_cli_pomodoro", package_name);
    }
}
