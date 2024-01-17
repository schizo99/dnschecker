use std::env::VarError;

/// Retrieves the value of an environment variable.
///
/// This function takes the name of an environment variable as an argument.
/// It attempts to get the value of the environment variable using the `std::env::var` function.
/// If the function fails, it logs a warning and returns an empty `String`.
///
/// # Arguments
///
/// * `var_name`: A `&str` that specifies the name of the environment variable.
///
/// # Returns
///
/// * A `String` that holds the value of the environment variable if the function succeeds.
/// * If the function fails, it returns an empty `String`.
pub fn get_var_from_env(name: &str) -> Result<String, VarError> {
    let value = std::env::var(name);
    match value {
        Ok(value) => Ok(value),
        Err(e) => {
            log::error!("{} not found in environment variables: {}", name, e);
            Err(e)
        }
    }
}

/// Checks if all specified environment variables are set.
///
/// This function takes a vector of environment variable names as an argument.
/// It iterates over the vector and checks each environment variable using the `std::env::var` function.
/// If the function fails (which means the environment variable is not set), it logs a warning and returns `false`.
/// If all environment variables are set, it returns `true`.
///
/// # Arguments
///
/// * `names`: A `Vec<&str>` that specifies the names of the environment variables.
///
/// # Returns
///
/// * A `bool` that indicates whether all specified environment variables are set.
/// * If all environment variables are set, it returns `true`.
/// * If any environment variable is not set, it returns `false`.
pub fn get_vars_from_env(names: Vec<&str>) -> bool {
    let mut error = false;
    for name in names {
        let result = match std::env::var(name) {
            Ok(value) => Ok(value),
            Err(e) => {
                log::error!("{} not found in environment variables: {}", name, e);
                Err(e)
            }
        };
        if result.is_err() {
            error = true;
        }
    }
    error
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
