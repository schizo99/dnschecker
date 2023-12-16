use std::time::Duration;
use serde_json::Value;
use std::fs::File;
use std::io::{Write, Read};
use chrono::{Local, DateTime};
use chrono::Duration as ChronoDuration;
use std::env;
use crate::vars;

pub fn send_telegram(token: &str, router_ip: &str, dns_ip: &str) -> bool {
    let lockfile = env::var("LOCKFILE").unwrap_or("/tmp/telegram.lock".to_string());
    let chat_id = vars::var("CHAT_ID");
    
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", &token);
    let text = format!("IP address mismatch between router and DNS server!\nRouter IP: {}\nDNS IP: {}", router_ip, dns_ip);
    let json = serde_json::json!({"chat_id": chat_id, "text": text, "disable_notification": false}); // Define the json variable
    let alarm_sent = read_timestamp_from_file(&lockfile);
    let timeout_duration = Duration::from_secs(10); // Set the timeout duration to 10 seconds
    if alarm_sent {
        log::info!("Alarm already sent: {:?}", !alarm_sent);
        false
    } else {
        log::info!("Sending alarm: {:?}", !alarm_sent);
        let response = client.post(&url).json(&json).timeout(timeout_duration).send();
        let response = match response {
            Ok(response) => response,
            Err(e) => {
                log::warn!("Failed to make HTTPS request: {:?}", e);
                return false;
            }
        };
        let response_text = response.text();
        let response_text = match response_text {
            Ok(response_text) => response_text,
            Err(e) => {
                log::warn!("Failed to get response text: {:?}", e);
                return false;
            }
        };

        let json: Value = match serde_json::from_str(&response_text) {
            Ok(json) => json,
            Err(e) => {
                log::warn!("Failed to parse JSON: {:?}", e);
                return false;
            }
        };
        let ok = json.get("ok").unwrap().as_bool().unwrap();
        ok
    }
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

pub fn read_timestamp_from_file(lockfile: &str) -> bool {
    if let Ok(mut file) = File::open(lockfile) {
        let mut contents = String::new();
        let readtimestamp = file.read_to_string(&mut contents);
        match readtimestamp {
            Ok(_) => log::info!("Timestamp read from file"),
            Err(e) => log::warn!("Failed to read timestamp from file: {:?}", e),
        }
        log::info!("Timestamp: {:?}", contents);

        if  let Ok(timestamp) = DateTime::parse_from_rfc2822(&contents) {
            let current = Local::now();
            if current.signed_duration_since(timestamp) < ChronoDuration::hours(24) {
                log::info!("Less than 24 hours since last alarm, not sending alarm!");
                true
            } else {
                create_timestamp(lockfile);
                log::info!("More than 24 hours since last alarm, sending alarm!");
                false
            }
        } else {
            log::info!("Failed to parse timestamp, creating new timestamp file");
            create_timestamp(lockfile);
            false
        }
    } else {
        log::info!("No lockfile found, creating new timestamp file");
        create_timestamp(lockfile);
        false
    }
}

