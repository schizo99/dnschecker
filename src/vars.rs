use std::env::VarError;

pub fn get_var_from_env(name: &str) -> Result<String, VarError> {
    let value = std::env::var(name);
    match value {
        Ok(value) => Ok(value),
        Err(e) => {
            log::error!("{} not found in environment variables: {}", name, e);
            return Err(e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var() {
        // Set an environment variable for the duration of the test
        std::env::set_var("TEST_VAR", "test value");

        // Call the function with the name of the environment variable
        let result = get_var_from_env("TEST_VAR").unwrap_or_else(|_| String::from(""));

        // Assert that the function returns the value of the environment variable
        assert_eq!(result, "test value");
        // Call the function with the name of the environment variable
        let result = get_var_from_env("NONEXIST").unwrap_or_else(|_| String::from(""));

        // Assert that the function returns the value of the environment variable
        assert_eq!(result, "");
    }
}
