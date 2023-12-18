use reqwest;
use serde_json::Value;
use std::time::Duration;
use crate::vars::get_var_from_env;

pub fn get_api() -> String{
    let username: String = get_var_from_env("API_KEY").unwrap();
    let password: String = get_var_from_env("API_SECRET").unwrap();
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

    parse_json(response_text)
}

fn parse_json(response_text: String) -> String {
    let json: Value = match serde_json::from_str(&response_text) {
        Ok(json) => json,
        Err(e) => {
            log::warn!("Failed to parse JSON: {:?}", e);
            return String::new();
        }
    };
    let value = json.get("igb3");
    let value = match value {
        Some(value) => value.get("ipv4"),
        None => {
            log::warn!("Failed to get \"igb3\" from JSON");
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

fn get_response(response: reqwest::blocking::Response) -> Result<String, String> {
    let response_text = response.text();
    let response_text = match response_text {
        Ok(response_text) => response_text,
        Err(e) => {
            log::warn!("Failed to get response text: {:?}", e);
            return Err(String::new());
        }
    };
    Ok(response_text)
}

fn call_endpoint(client: reqwest::blocking::Client, url: String, username: String, password: String) -> Result<reqwest::blocking::Response, String> {
    let timeout_duration = Duration::from_secs(10);
    let response = client.get(&url).basic_auth(username, Some(password)).timeout(timeout_duration).send();
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            log::warn!("Failed to make HTTPS request: {:?}", e);
            return Err(String::new());
        }
    };
    Ok(response)
}

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
            then.status(200).body("{\"igb3\": {\"ipv4\": [{\"ipaddr\": \"127.0.0.1\"}]}}");
        });
        std::env::set_var("API_KEY", "username");
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
        // Call the function with a JSON string that has the expected structure
        let result = parse_json(String::from("{\"igb3\": {\"ipv4\": [{\"ipaddr\": \"192.168.1.1\"}]}}"));

        // Assert that the function returns the expected output
        assert_eq!(result, "192.168.1.1");

        // Call the function with a JSON string that does not have the expected structure
        let result = parse_json(String::from("{\"foo\": \"bar\"}"));

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
        let result = call_endpoint(client, server.url("/test"), String::from("username"), String::from("password")).unwrap();
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
        let result = call_endpoint(client, server.url("/test"), String::from("username"), String::from("password")); // Add timeout_duration argument

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