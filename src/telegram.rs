use crate::vars::get_var_from_env;
use chrono::Duration as ChronoDuration;
use chrono::{DateTime, Local};
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::time::Duration;

pub fn send_telegram(token: &str, router_ip: &str, dns_ip: &str) -> bool {
    let lockfile = env::var("LOCKFILE").unwrap_or("/tmp/telegram.lock".to_string());
    let chat_id = get_var_from_env("CHAT_ID").unwrap();

    let url = format!("https://api.telegram.org/bot{}/sendMessage", &token);
    let text = format!(
        "IP address mismatch between router and DNS server!\nRouter IP: {}\nDNS IP: {}",
        router_ip, dns_ip
    );
    let json = serde_json::json!({"chat_id": chat_id, "text": text, "disable_notification": false}); // Define the json variable
    let alarm_sent = read_timestamp_from_file(&lockfile);

    if alarm_sent {
        log::info!("Alarm already sent: {:?}", !alarm_sent);
        if router_ip == dns_ip {
            log::debug!("IP addresses are the same again, resetting alarm");
            match reset_alarm(&lockfile, &chat_id, url) {
                Ok(value) => value,
                Err(value) => value,
            };
            return true;
        }
        false
    } else {
        if router_ip != dns_ip {
            log::info!("Sending alarm: {:?}", !alarm_sent);
            let response = match do_request(url, json) {
                Ok(value) => value,
                Err(value) => return value,
            };
            let response_text = match parse_response(response) {
                Ok(value) => value,
                Err(value) => return value,
            };
            create_timestamp(&lockfile);
            parse_json(response_text)
        } else {
            log::trace!("IP addresses are the same, not sending alarm");
            true
        }
    }
}

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

fn do_request(url: String, json: Value) -> Result<reqwest::blocking::Response, bool> {
    let client = reqwest::blocking::Client::new();
    let timeout_duration = Duration::from_secs(10); // Set the timeout duration to 10 seconds
    let response = client
        .post(&url)
        .json(&json)
        .timeout(timeout_duration)
        .send();
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            log::warn!("Failed to make HTTPS request: {:?}", e);
            return Err(false);
        }
    };
    Ok(response)
}

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

fn reset_lockfile(lockfile: &str) -> Result<String, String> {
    match std::fs::remove_file(lockfile) {
        Ok(_) => Ok("Lockfile reset".to_string()),
        Err(e) => Err(format!("Failed to reset lockfile: {:?}", e)),
    }
}

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
            if current.signed_duration_since(timestamp) < ChronoDuration::hours(24) {
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
        assert!(current.signed_duration_since(timestamp) < chrono::Duration::minutes(1));
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
