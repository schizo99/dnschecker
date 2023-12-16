use std::env;
use std::thread;
use std::time::Duration;
mod api;
mod dns;
mod telegram;

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    simple_logger::init_with_env().unwrap();
    log::info!("Starting DNS checker");
    let token = env::var("TELEGRAM_BOT_TOKEN");
    let token = match token {
        Ok(token) => token,
        Err(e) => {
            log::error!(
                "TELEGRAM_BOT_TOKEN not found in environment variables: {}",
                e
            );
            return;
        }
    };
    let hostname = env::var("DNS_HOSTNAME");
    let hostname = match hostname {
        Ok(hostname) => hostname,
        Err(e) => {
            log::error!("DNS_HOSTNAME not found in environment variables: {}", e);
            return;
        }
    };

    loop {
        let ip_address = dns::resolve_hostname(&hostname);
        if ip_address.is_empty() {
            log::warn!("Failed to get IP address");
        }
        let wan_ip = api::get_api();
        if wan_ip.is_empty() {
            log::warn!("Failed to get WAN IP address");
        }

        log::info!("The IP address of {} is: {:?}", hostname, ip_address);
        log::info!("The WAN IP address of is: {:?}", wan_ip);
        if ip_address.is_empty() || wan_ip.is_empty() {
            log::warn!("Since one of the IP addresses is empty, skipping comparison");
        } else {
            if ip_address == wan_ip {
                log::info!("IP address is the same");
            } else {
                log::info!("IP address is different");
                if !telegram::send_telegram(&token, &ip_address, &wan_ip) {
                    log::warn!("Failed to send telegram");
                } else {
                    log::info!("Telegram sent");
                }
            }
        }
        log::info!("Sleeping for 30 minutes");
        thread::sleep(Duration::from_secs(1800)); // Sleep for 30 minutes
    }
}
