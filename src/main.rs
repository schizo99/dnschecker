use std::thread;
use std::time::Duration;
mod api;
mod dns;
mod telegram;

mod vars;
use crate::vars::*;

fn main() {
    init();
    let (hostname, token) = verify_env_vars();
    let mut counter: i32 = 1;
    loop {
        counter = verify_ips(&hostname, &token, counter);
    }
}

fn verify_env_vars() -> (String, String) {
    let envvars: Vec<&str> = vec![
        "TELEGRAM_TOKEN",
        "DNS_HOSTNAME",
        "API_KEY",
        "API_SECRET",
        "URL",
        "CHAT_ID",
        "INTERFACE",
    ];

    let error: bool = get_vars_from_env(envvars);
    if error {
        log::error!("One or more environment variables are missing");
        std::process::exit(1);
    }
    let token: String =
        get_var_from_env("TELEGRAM_TOKEN").unwrap_or_else(|_| std::process::exit(1));
    let hostname: String =
        get_var_from_env("DNS_HOSTNAME").unwrap_or_else(|_| std::process::exit(1));

    (hostname, token)
}

fn verify_ips(hostname: &String, token: &String, counter: i32) -> i32 {
    if counter == 0 {
        log::info!("Verifying IPs");
    }
    let ip_address = dns::resolve_hostname(hostname);
    if ip_address.is_empty() {
        log::warn!("Failed to get IP address");
    }
    let wan_ip = api::get_api();
    if wan_ip.is_empty() {
        log::warn!("Failed to get WAN IP address");
    }

    log::debug!(
        "The IP address of {} is: {}, WAN IP address is: {}",
        hostname,
        ip_address,
        wan_ip
    );
    if ip_address.is_empty() || wan_ip.is_empty() {
        log::warn!("Since one of the IP addresses is empty, skipping comparison");
    } else {
        if ip_address != wan_ip {
            log::info!("IP address is different");
            if !token.is_empty() {
                if !telegram::send_telegram(token, &ip_address, &wan_ip) {
                    log::warn!("Failed to send telegram");
                } else {
                    log::info!("Telegram sent");
                }
            }
        } else {
            if !telegram::send_telegram(token, &ip_address, &wan_ip) {
                log::warn!("Failed to send successful update telegram");
            }
        }
    }
    log::debug!("Sleeping for 10 seconds");
    thread::sleep(Duration::from_secs(10));
    let counter: i32 = counter + 1;
    if counter >= 180 {
        log::info!("30 minutes passed");
        return 1;
    }
    counter
}

fn init() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    simple_logger::init_with_env().unwrap();
    log::info!("Starting DNS checker");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        // Call the function
        init();

        // Assert that the function sets the "RUST_LOG" environment variable to "INFO"
        assert_eq!(std::env::var("RUST_LOG").unwrap(), "INFO");
    }
}
