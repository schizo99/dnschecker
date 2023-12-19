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
    loop {
        verify_ips(&hostname, &token);
    }
}
fn verify_env_vars() -> (String, String) {
    let token = get_var_from_env("TELEGRAM_TOKEN").unwrap_or_else(|_| std::process::exit(1));
    let hostname = get_var_from_env("DNS_HOSTNAME").unwrap_or_else(|_| std::process::exit(1));

    log::debug!("TELEGRAM_TOKEN: {}", token);
    log::debug!("DNS_HOSTNAME: {}", hostname);
    log::debug!("API_KEY: {}",  get_var_from_env("API_KEY").unwrap_or_else(|_| std::process::exit(1)));
    log::debug!("API_SECRET: {}", get_var_from_env("API_SECRET").unwrap_or_else(|_| std::process::exit(1)));
    log::debug!("URL: {}", get_var_from_env("URL").unwrap_or_else(|_| std::process::exit(1)));
    log::debug!("CHAT_ID: {}", get_var_from_env("CHAT_ID").unwrap_or_else(|_| std::process::exit(1)));

    (token, hostname)
}

fn verify_ips(hostname: &String, token: &String) {
    let ip_address = dns::resolve_hostname(hostname);
    if ip_address.is_empty() {
        log::warn!("Failed to get IP address");
    }
    let wan_ip = api::get_api();
    if wan_ip.is_empty() {
        log::warn!("Failed to get WAN IP address");
    }

    log::info!(
        "The IP address of {} is: {}, WAP IP address is: {}",
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
        }
    }
    log::info!("Sleeping for 30 minutes");
    thread::sleep(Duration::from_secs(1800));
    // Sleep for 30 minutes
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
