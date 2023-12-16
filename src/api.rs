use reqwest;
use serde_json::Value;
use std::time::Duration;

use crate::vars;

pub fn get_api() -> String{
    let username = vars::var("API_KEY");
    let password = vars::var("API_SECRET");
    let url = vars::var("URL");

    let mut client_builder = reqwest::blocking::Client::builder();
    client_builder = client_builder.danger_accept_invalid_certs(true);
    let client = client_builder.build();
    let client = match client {
        Ok(client) => client,
        Err(err) => {
            log::warn!("Failed to build client: {}", err);
            return String::new();
        }
    };
    let timeout_duration = Duration::from_secs(10);
    let response = client.get(&url).basic_auth(username, Some(password)).timeout(timeout_duration).send();
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            log::warn!("Failed to make HTTPS request: {:?}", e);
            return String::new();
        }
    };
    let response_text = response.text();
    let response_text = match response_text {
        Ok(response_text) => response_text,
        Err(e) => {
            log::warn!("Failed to get response text: {:?}", e);
            return String::new();
        }
    };

    let json: Value = match serde_json::from_str(&response_text) {
        Ok(json) => json,
        Err(e) => {
            log::warn!("Failed to parse JSON: {:?}", e);
            return String::new();
        }
    };
    let value = json.get("igb3").unwrap().get("ipv4").unwrap();
    let value = value.get(0).unwrap().get("ipaddr").unwrap();
    value.as_str().unwrap().to_string()

}
