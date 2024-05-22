use crate::vars::get_var_from_env;
use chrono::Duration as ChronoDuration;
use chrono::{DateTime, Local};
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::time::Duration;

/// Sends a message to a Telegram chat when there is an IP address mismatch between the router and the DNS server.
///
/// This function takes a Telegram bot token, a router IP address, and a DNS server IP address as arguments.
/// It first retrieves the lockfile path and chat ID from environment variables.
/// It then constructs the URL for the Telegram API and the text of the message.
/// It checks if an alarm has already been sent by reading the timestamp from the lockfile.
/// If an alarm has already been sent and the IP addresses are the same again, it resets the alarm.
/// If an alarm has not been sent and the IP addresses are different, it sends an alarm.
///
/// # Arguments
///
/// * `token`: A `&str` that specifies the Telegram bot token.
/// * `router_ip`: A `&str` that specifies the router IP address.
/// * `dns_ip`: A `&str` that specifies the DNS server IP address.
///
/// # Returns
///
/// * A `bool` that indicates whether the function succeeded.
/// * If the function succeeds, it returns `true`.
/// * If the function fails, it returns `false`.
pub fn send_telegram(token: &str, router_ip: &str, dns_ip: &str) -> bool {
    let lockfile = env::var("LOCKFILE").unwrap_or("/tmp/telegram.lock".to_string());
    let chat_id = get_var_from_env("CHAT_ID").unwrap();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", &token);
    let text = format!(
        "IP address mismatch between router and DNS server!\nRouter IP: {}\nDNS IP: {}",
        router_ip, dns_ip
    );
    let json = serde_json::json!({"chat_id": chat_id, "text": text, "disable_notification": false});

    let alarm_sent = read_timestamp_from_file(&lockfile);

    if alarm_sent && router_ip == dns_ip {
        log::debug!("IP addresses are the same again, resetting alarm");
        reset_alarm(&lockfile, &chat_id, url).is_ok()
    } else if !alarm_sent && router_ip != dns_ip {
        log::info!("Sending alarm");
        if let Ok(response) = do_request(url, json) {
            if let Ok(response_text) = parse_response(response) {
                log::debug!("Creating timestamp");
                create_timestamp(&lockfile);
                log::debug!("Parsing response");
                parse_json(response_text)
            } else {
                log::error!("Failed to parse response");
                false
            }
        } else {
            log::error!("Failed to send alarm");
            false
        }
    } else {
        log::trace!("IP addresses are the same, not sending alarm");
        true
    }
}

/// Sends a reset message to a Telegram chat when the IP addresses of the router and the DNS server are the same again.
///
/// This function takes the lockfile path, chat ID, and the URL for the Telegram API as arguments.
/// It first constructs the JSON payload for the Telegram API request, which includes the chat ID, the text of the message, and a flag to disable notification.
/// It then sends the request to the Telegram API using the `do_request` function.
/// If the function fails, it logs a warning and returns an `Err` with a message.
///
/// It then parses the response from the Telegram API using the `parse_response` function.
/// If the function fails, it logs a warning and returns an `Err` with a message.
///
/// It then parses the JSON response from the Telegram API using the `parse_json` function.
/// If the function fails, it logs a warning and returns an `Err` with a message.
///
/// If the function succeeds, it logs an info message, resets the lockfile using the `reset_lockfile` function, and returns an `Ok` with a message.
///
/// # Arguments
///
/// * `lockfile`: A `&str` that specifies the lockfile path.
/// * `chat_id`: A `&str` that specifies the chat ID.
/// * `url`: A `String` that specifies the URL for the Telegram API.
///
/// # Returns
///
/// * A `Result<String, String>` that holds a message if the function succeeds.
/// * If the function fails, it returns an `Err` with a message.
fn reset_alarm(lockfile: &str, chat_id: &str, url: String) -> Result<String, String> {
    let json = serde_json::json!({"chat_id": chat_id, "text": "IP addresses are the same again", "disable_notification": false}); // Define the json variable
    let response = match do_request(url, json) {
        Ok(value) => value,
        Err(_) => return Err("failed to send reset alarm".to_string()),
    };
    let response_text = match parse_response(response) {
        Ok(value) => value,
        Err(_) => return Err("failed to parse response".to_string()),
    };
    let result = parse_json(response_text);
    if result {
        log::info!("Alarm has been reset");
        match reset_lockfile(lockfile) {
            Ok(value) => value,
            Err(value) => return Err(value.into()),
        };
        Ok("Alarm has been reset".to_string())
    } else {
        log::warn!("Failed to reset alarm");
        Err("Failed to reset alarm".to_string())
    }
}

/// Parses a JSON string and extracts the value of the "ok" field.
///
/// This function takes a JSON string as an argument.
/// It attempts to parse the JSON string into a `serde_json::Value` using the `serde_json::from_str` function.
/// If the function fails, it logs a warning and returns `false`.
///
/// It then attempts to get the value of the "ok" field from the `serde_json::Value` using the `Value::get` method.
/// If the function fails, it logs a warning and returns `false`.
///
/// It then attempts to convert the value of the "ok" field to a `bool` using the `Value::as_bool` method.
/// If the function fails, it logs a warning and returns `false`.
///
/// If all steps succeed, it returns the value of the "ok" field as a `bool`.
///
/// # Arguments
///
/// * `response_text`: A `String` that specifies the JSON string to parse.
///
/// # Returns
///
/// * A `bool` that holds the value of the "ok" field if the function succeeds.
/// * If any step fails, it returns `false`.
fn parse_json(response_text: String) -> bool {
    let json: Value = match serde_json::from_str(&response_text) {
        Ok(json) => json,
        Err(e) => {
            log::warn!("Failed to parse JSON: {:?}", e);
            return false;
        }
    };
    let ok = json.get("ok");
    let ok = match ok {
        Some(ok) => ok.as_bool().unwrap(),
        None => {
            log::warn!("Failed to get \"ok\" from JSON");
            return false;
        }
    };

    ok
}

/// Extracts the text from an HTTP response.
///
/// This function takes an HTTP response as an argument.
/// It attempts to extract the text from the HTTP response using the `reqwest::blocking::Response::text` method.
/// If the method fails, it logs a warning and returns an `Err` with `false`.
///
/// If the method succeeds, it returns an `Ok` with the text of the HTTP response.
///
/// # Arguments
///
/// * `response`: A `reqwest::blocking::Response` that specifies the HTTP response to extract the text from.
///
/// # Returns
///
/// * A `Result<String, bool>` that holds the text of the HTTP response if the function succeeds.
/// * If the function fails, it returns an `Err` with `false`.
fn parse_response(response: reqwest::blocking::Response) -> Result<String, bool> {
    let response_text = response.text();
    let response_text = match response_text {
        Ok(response_text) => response_text,
        Err(e) => {
            log::warn!("Failed to get response text: {:?}", e);
            return Err(false);
        }
    };
    Ok(response_text)
}

/// Makes an HTTP POST request with a JSON payload.
///
/// This function takes a URL and a JSON value as arguments.
/// It first creates a new `reqwest::blocking::Client`.
/// It then sets the timeout duration for the request to 10 seconds.
/// It then attempts to make the HTTP POST request using the `reqwest::blocking::Client::post` method, the `RequestBuilder::json` method to set the JSON payload, the `RequestBuilder::timeout` method to set the timeout duration, and the `RequestBuilder::send` method to send the request.
/// If the method fails, it logs a warning and returns an `Err` with `false`.
///
/// If the method succeeds, it returns an `Ok` with the HTTP response.
///
/// # Arguments
///
/// * `url`: A `String` that specifies the URL to make the HTTP POST request to.
/// * `json`: A `serde_json::Value` that specifies the JSON payload for the HTTP POST request.
///
/// # Returns
///
/// * A `Result<reqwest::blocking::Response, bool>` that holds the HTTP response if the function succeeds.
/// * If the function fails, it returns an `Err` with `false`.
fn do_request(url: String, json: Value) -> Result<reqwest::blocking::Response, bool> {
    let client = reqwest::blocking::Client::new();
    let timeout_duration = Duration::from_secs(10); // Set the timeout duration to 10 seconds
    let request = client
        .post(url)
        .json(&json)
        .timeout(timeout_duration)
        .build()
        .map_err(|e| {
            log::warn!("Failed to build request: {:?}", e);
            true
        })?;
    let response = client
        .execute(request)
        .map_err(|e| {
            log::warn!("Failed to make HTTPS request: {:?}", e);
            true
        })?;
    Ok(response)
}

/// Creates a timestamp and writes it to a lockfile.
///
/// This function takes a lockfile path as an argument.
/// It first creates a new file at the lockfile path using the `File::create` method.
/// It then sets the length of the file to 0 using the `File::set_len` method to ensure that the file is empty.
/// It then creates a timestamp using the `DateTime::to_rfc2822` method and the current local time.
/// It then writes the timestamp to the file using the `Write::write_all` method.
///
/// If any step fails, it logs a warning.
/// If all steps succeed, it logs an info message.
///
/// # Arguments
///
/// * `lockfile`: A `&str` that specifies the lockfile path.
fn create_timestamp(lockfile: &str) {
    let mut file = File::create(lockfile).unwrap();
    match file.set_len(0) {
        Ok(_) => log::info!("Lockfile created"),
        Err(e) => log::warn!("Failed to create lockfile: {:?}", e),
    }
    let timestamp = DateTime::to_rfc2822(&Local::now());
    let written = file.write_all(timestamp.as_bytes());
    match written {
        Ok(_) => log::info!("Timestamp written to file"),
        Err(e) => log::warn!("Failed to write timestamp to file: {:?}", e),
    }
}

/// Removes a lockfile.
///
/// This function takes a lockfile path as an argument.
/// It attempts to remove the file at the lockfile path using the `std::fs::remove_file` function.
/// If the function fails, it logs a warning and returns an `Err` with a message.
///
/// If the function succeeds, it logs an info message and returns an `Ok` with a message.
///
/// # Arguments
///
/// * `lockfile`: A `&str` that specifies the lockfile path.
///
/// # Returns
///
/// * A `Result<String, String>` that holds a message if the function succeeds.
/// * If the function fails, it returns an `Err` with a message.
fn reset_lockfile(lockfile: &str) -> Result<String, String> {
    match std::fs::remove_file(lockfile) {
        Ok(_) => Ok("Lockfile reset".to_string()),
        Err(e) => Err(format!("Failed to reset lockfile: {:?}", e)),
    }
}

/// Reads a timestamp from a lockfile and checks if it's less than 24 hours old.
///
/// This function takes a lockfile path as an argument.
/// It first attempts to open the file at the lockfile path using the `File::open` method.
/// If the method fails, it logs a warning and returns `false`.
///
/// It then reads the contents of the file into a `String` using the `Read::read_to_string` method.
/// If the method fails, it logs a warning and returns `false`.
///
/// It then attempts to parse the contents of the file into a `DateTime` using the `DateTime::parse_from_rfc2822` method.
/// If the method fails, it logs a warning and returns `false`.
///
/// It then gets the current local time and checks if the duration since the timestamp is less than 24 hours.
/// If it is, it logs an info message and returns `true`.
/// If it's not, it logs an info message and returns `false`.
///
/// # Arguments
///
/// * `lockfile`: A `&str` that specifies the lockfile path.
///
/// # Returns
///
/// * A `bool` that indicates whether the timestamp is less than 24 hours old.
pub fn read_timestamp_from_file(lockfile: &str) -> bool {
    if let Ok(mut file) = File::open(lockfile) {
        let mut contents = String::new();
        let readtimestamp = file.read_to_string(&mut contents);
        match readtimestamp {
            Ok(_) => log::info!("Timestamp read from file"),
            Err(e) => log::warn!("Failed to read timestamp from file: {:?}", e),
        }
        log::info!("Timestamp: {:?}", contents);

        if let Ok(timestamp) = DateTime::parse_from_rfc2822(&contents) {
            let current = Local::now();
            if current.signed_duration_since(timestamp) < ChronoDuration::try_hours(24).unwrap() {
                log::info!("Less than 24 hours since last alarm, not sending alarm!");
                true
            } else {
                log::info!("More than 24 hours since last alarm, sending alarm!");
                false
            }
        } else {
            log::info!("Failed to parse timestamp, creating new timestamp file");
            false
        }
    } else {
        log::debug!("No lockfile found, alarm not previously sent");
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::offset::TimeZone;
    use httpmock::MockServer;
    use std::fs::File;
    use std::io::Write;
    #[test]
    fn test_parse_response() {
        let server = MockServer::start();

        // Create a mock for the endpoint
        let mock = server.mock(|when, then| {
            when.method("POST").path("/get");
            then.status(200).body("test response");
        });

        let json =
            serde_json::json!({"chat_id": "111", "text": "text", "disable_notification": false}); // Define the json variable
        let response = do_request(server.url("/get"), json).unwrap();

        // Call the function with the Response object
        let result = parse_response(response);

        // Assert that the function returns the expected output
        assert_eq!(result.unwrap(), "test response");
        mock.assert();
    }

    #[test]
    fn test_do_request() {
        let server = MockServer::start();

        // Create a mock for the endpoint
        let mock = server.mock(|when, then| {
            when.method("POST").path("/get");
            then.status(403).body("test response");
        });

        let json =
            serde_json::json!({"chat_id": "111", "text": "text", "disable_notification": false}); // Define the json variable
        let response = do_request(server.url("/get"), json).unwrap();

        // Call the function with the Response object
        let result = parse_response(response);

        // Assert that the function returns the expected output
        assert_eq!(result.unwrap(), "test response");
        mock.assert();
    }

    #[test]
    fn test_parse_json() {
        // Call the function with a JSON string that has "ok": true
        let result = parse_json(String::from("{\"ok\": true}"));

        // Assert that the function returns true
        assert_eq!(result, true);

        // Call the function with a JSON string that has "ok": false
        let result = parse_json(String::from("{\"ok\": false}"));

        // Assert that the function returns false
        assert_eq!(result, false);

        // Call the function with a JSON string that does not have "ok"
        let result = parse_json(String::from("{\"foo\": \"bar\"}"));

        // Assert that the function returns false
        assert_eq!(result, false);

        // Call the function with a string that is not valid JSON
        let result = parse_json(String::from("not valid JSON"));

        // Assert that the function returns false
        assert_eq!(result, false);
    }

    #[test]
    fn test_create_timestamp() {
        // Create a temporary file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();

        // Call the function with the temporary file
        create_timestamp(&file_path);

        // Open the file and read its contents
        let mut file = File::open(&file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        // Check if the contents can be parsed as a timestamp
        let parsed = DateTime::parse_from_rfc2822(&contents);
        assert!(parsed.is_ok());

        // Check if the timestamp is recent (within the last minute)
        let timestamp = parsed.unwrap();
        let current = Local::now();
        assert!(
            current.signed_duration_since(timestamp) < chrono::Duration::try_minutes(1).unwrap()
        );
    }

    #[test]
    fn test_read_timestamp_from_file() {
        // Create a temporary file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();

        // Write a known timestamp to the file (older than 24 hours)
        let mut file = File::create(&file_path).unwrap();
        let timestamp_old = chrono::Local
            .with_ymd_and_hms(2022, 1, 1, 0, 0, 0)
            .unwrap()
            .to_rfc2822();
        writeln!(file, "{}", timestamp_old).unwrap();

        // Call the function with the temporary file
        let result_old = read_timestamp_from_file(&file_path);

        // Assert that the function returns false (because the timestamp is more than 24 hours ago)
        assert_eq!(result_old, false);

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();

        // Write a known timestamp to the file (older than 24 hours)
        let mut file = File::create(&file_path).unwrap();

        let timestamp = DateTime::to_rfc2822(&Local::now());
        file.write_all(timestamp.as_bytes()).unwrap();

        // Call the function with the temporary file again
        let result_new = read_timestamp_from_file(&file_path);

        // Assert that the function returns true (because the timestamp is within 24 hours)
        assert_eq!(result_new, true);
    }
}
