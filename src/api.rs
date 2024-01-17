use crate::vars::get_var_from_env;
use reqwest;
use serde_json::Value;
use std::time::Duration;

/// Makes an API request and parses the response.
///
/// This function retrieves the values of the "API_KEY", "API_SECRET", "INTERFACE", and "URL" environment variables.
/// It then builds a `reqwest::Client` and makes a request to the endpoint specified by the "URL" environment variable.
/// The response is then parsed into a JSON object.
/// The function then retrieves the "ipv4" field of the object specified by the "INTERFACE" environment variable from the JSON object.
///
/// # Returns
///
/// * A `String` that holds the value of the "ipv4" field of the object specified by the "INTERFACE" environment variable.
/// * If any step fails, it returns an empty `String`.
pub fn get_api() -> String {
    let username: String = get_var_from_env("API_KEY").unwrap();
    let password: String = get_var_from_env("API_SECRET").unwrap();
    let interface: String = get_var_from_env("INTERFACE").unwrap();
    let url: String = get_var_from_env("URL").unwrap();

    let client = match build_client() {
        Ok(value) => value,
        Err(value) => return value,
    };
    let response = match call_endpoint(client, url, username, password) {
        Ok(value) => value,
        Err(value) => return value,
    };
    let response_text = match get_response(response) {
        Ok(value) => value,
        Err(value) => return value,
    };

    parse_json(response_text, &interface)
}

/// Parses a JSON string and extracts a specific value from it.
///
/// This function takes a JSON string and the name of an interface as arguments.
/// It attempts to parse the JSON string into a `serde_json::Value` object using the `serde_json::from_str` function.
/// If the parsing fails, it logs a warning and returns an empty `String`.
///
/// It then attempts to get the value of the object specified by the interface from the `serde_json::Value` object.
/// If the object does not exist, it logs a warning and returns an empty `String`.
///
/// Finally, it attempts to get the "ipv4" field of the object.
/// If the "ipv4" field does not exist, it logs a warning and returns an empty `String`.
/// If the "ipv4" field exists, it returns its value as a `String`.
///
/// # Arguments
///
/// * `response_text`: A `String` that holds the JSON string to parse.
/// * `interface`: A `&str` that specifies the name of the interface to get the value from.
///
/// # Returns
///
/// * A `String` that holds the value of the "ipv4" field of the object specified by the interface.
/// * If any step fails, it returns an empty `String`.
fn parse_json(response_text: String, interface: &str) -> String {
    let json: Value = match serde_json::from_str(&response_text) {
        Ok(json) => json,
        Err(e) => {
            log::warn!("Failed to parse JSON: {:?}", e);
            return String::new();
        }
    };
    let value = json.get(interface);
    let value = match value {
        Some(value) => value.get("ipv4"),
        None => {
            log::warn!("Failed to get \"{}\" from JSON", interface);
            return String::new();
        }
    };
    let value = match value {
        Some(value) => value,
        None => {
            log::warn!("Failed to get \"ipv4\" from JSON");
            return String::new();
        }
    };
    let value = value.get(0).unwrap().get("ipaddr").unwrap();
    value.as_str().unwrap().to_string()
}

/// Extracts the body of an HTTP response as a string.
///
/// This function takes a `reqwest::blocking::Response` object as an argument.
/// It attempts to get the body of the response as a string using the `reqwest::blocking::Response::text` method.
/// If the method fails, it logs a warning and returns an `Err` with an empty `String`.
///
/// # Arguments
///
/// * `response`: A `reqwest::blocking::Response` object that represents the HTTP response.
///
/// # Returns
///
/// * A `Result<String, String>` that holds the body of the response as a `String` if the method succeeds.
/// * If the method fails, it returns an `Err` with an empty `String`.
fn get_response(response: reqwest::blocking::Response) -> Result<String, String> {
    let response_text = response.text();
    let response_text = match response_text {
        Ok(response_text) => response_text,
        Err(e) => {
            log::warn!("Failed to get response text: {}", e);
            return Err(String::new());
        }
    };
    Ok(response_text)
}

/// Makes an HTTP request to a specified endpoint and returns the response.
///
/// This function takes a `reqwest::blocking::Client`, a URL, a username, and a password as arguments.
/// It sets a timeout of 10 seconds for the request using the `reqwest::blocking::RequestBuilder::timeout` method.
/// It then makes a GET request to the specified URL using the `reqwest::blocking::RequestBuilder::get` method.
/// It sets the username and password for basic authentication using the `reqwest::blocking::RequestBuilder::basic_auth` method.
/// It sends the request and gets the response using the `reqwest::blocking::RequestBuilder::send` method.
/// If the method fails, it logs a warning and returns an `Err` with an empty `String`.
///
/// # Arguments
///
/// * `client`: A `reqwest::blocking::Client` that is used to make the request.
/// * `url`: A `String` that specifies the URL of the endpoint.
/// * `username`: A `String` that specifies the username for basic authentication.
/// * `password`: A `String` that specifies the password for basic authentication.
///
/// # Returns
///
/// * A `Result<reqwest::blocking::Response, String>` that holds the response if the request succeeds.
/// * If the request fails, it returns an `Err` with an empty `String`.
fn call_endpoint(
    client: reqwest::blocking::Client,
    url: String,
    username: String,
    password: String,
) -> Result<reqwest::blocking::Response, String> {
    let timeout_duration = Duration::from_secs(10);
    let response = client
        .get(&url)
        .basic_auth(username, Some(password))
        .timeout(timeout_duration)
        .send();
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            log::warn!("Failed to make HTTPS request: {}", e);
            return Err(String::new());
        }
    };
    Ok(response)
}

/// Creates a new `reqwest::blocking::Client` instance with certain configurations.
///
/// This function first creates a `reqwest::blocking::ClientBuilder` instance using the `reqwest::blocking::Client::builder` method.
/// It then configures the builder to accept invalid certificates using the `reqwest::blocking::ClientBuilder::danger_accept_invalid_certs` method.
/// It builds the `reqwest::blocking::Client` instance using the `reqwest::blocking::ClientBuilder::build` method.
/// If the method fails, it logs a warning and returns an `Err` with an empty `String`.
///
/// # Returns
///
/// * A `Result<reqwest::blocking::Client, String>` that holds the `reqwest::blocking::Client` instance if the method succeeds.
/// * If the method fails, it returns an `Err` with an empty `String`.
fn build_client() -> Result<reqwest::blocking::Client, String> {
    let mut client_builder = reqwest::blocking::Client::builder();
    client_builder = client_builder.danger_accept_invalid_certs(true);
    let client = client_builder.build();
    let client = match client {
        Ok(client) => client,
        Err(err) => {
            log::warn!("Failed to build client: {}", err);
            return Err(String::new());
        }
    };
    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::MockServer;

    #[test]
    fn test_get_api() {
        let server = MockServer::start();

        // Create a mock for the endpoint
        let mock = server.mock(|when, then| {
            when.method("GET").path("/test");
            then.status(200)
                .body("{\"igb3\": {\"ipv4\": [{\"ipaddr\": \"127.0.0.1\"}]}}");
        });
        std::env::set_var("API_KEY", "username");
        std::env::set_var("INTERFACE", "igb3");
        std::env::set_var("API_SECRET", "password");
        std::env::set_var("URL", server.url("/test"));
        // Call the function with the mock server's URL
        let result = get_api();

        // Assert that the function returns the expected output
        assert_eq!(result, "127.0.0.1");

        // Assert that the mock was called
        mock.assert();
    }

    #[test]
    fn test_parse_json() {
        let interface = "igb3";
        // Call the function with a JSON string that has the expected structure
        let result = parse_json(
            String::from("{\"igb3\": {\"ipv4\": [{\"ipaddr\": \"192.168.1.1\"}]}}"),
            interface,
        );

        // Assert that the function returns the expected output
        assert_eq!(result, "192.168.1.1");
        // Call the function with a JSON string that does not have the expected structure
        let result = parse_json(String::from("{\"foo\": \"bar\"}"), interface);

        // Assert that the function returns an empty string
        assert_eq!(result, "");
    }
    #[test]
    fn test_get_response() {
        let server = MockServer::start();

        // Create a mock for the endpoint
        let mock = server.mock(|when, then| {
            when.method("GET").path("/test");
            then.status(200).body("OK");
        });

        // Call the function with the mock server's URL
        let client = reqwest::blocking::Client::new();
        let result = call_endpoint(
            client,
            server.url("/test"),
            String::from("username"),
            String::from("password"),
        )
        .unwrap();
        let result = get_response(result);

        // Assert that the function returns the expected output
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response, "OK");

        // Assert that the mock was called
        mock.assert();
    }
    #[test]
    fn test_call_endpoint() {
        // Start a mock server
        let server = MockServer::start();

        // Create a mock for the endpoint
        let mock = server.mock(|when, then| {
            when.method("GET").path("/test");
            then.status(200).body("OK");
        });

        // Call the function with the mock server's URL
        let client = reqwest::blocking::Client::new();
        let result = call_endpoint(
            client,
            server.url("/test"),
            String::from("username"),
            String::from("password"),
        ); // Add timeout_duration argument

        // Assert that the function returns the expected output
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), 200);

        // Assert that the mock was called
        mock.assert();
    }

    #[test]
    fn test_build_client() {
        // Call the function
        let result = build_client();

        // Assert that the function returns a client
        assert!(result.is_ok());
    }
}
