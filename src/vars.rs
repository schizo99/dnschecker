pub fn var(name: &str) -> String {
    let value = std::env::var(name);
    match value {
        Ok(value) => value,
        Err(e) => {
            log::error!("{} not found in environment variables: {}", name, e);
            std::process::exit(1);
        }
    }
}
