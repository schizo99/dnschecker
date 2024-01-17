use shuteye::sleep;
use std::time::Duration;
mod api;
mod dns;
mod telegram;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod vars;
use crate::vars::*;

fn main() {
    let sig_received = Arc::new(AtomicBool::new(false));
    let r = sig_received.clone();

    // Set the SIGTERM and SIGINT handler
    let mut signals = Signals::new(&[SIGTERM, SIGINT]).unwrap();
    std::thread::spawn(move || {
        for _ in signals.forever() {
            r.store(true, Ordering::SeqCst);
        }
    });
    init();
    let (hostname, token) = verify_env_vars();
    let mut counter: i32 = 1;
    loop {
        counter = verify_ips(&hostname, &token, counter);
        if sig_received.load(Ordering::SeqCst) {
            break;
        }
    }
}

/// Verifies the presence of certain environment variables and retrieves their values.
///
/// This function checks if the following environment variables are set:
/// "TELEGRAM_TOKEN", "DNS_HOSTNAME", "API_KEY", "API_SECRET", "URL", "CHAT_ID", "INTERFACE".
///
/// It does this by calling the `get_vars_from_env` function with a vector of these variable names.
/// If any of these variables are not set (indicated by `get_vars_from_env` returning true),
/// it logs an error message and exits the program with a status code of 1.
///
/// If all variables are set, it retrieves the values of "TELEGRAM_TOKEN" and "DNS_HOSTNAME"
/// using the `get_var_from_env` function and returns them.
///
/// # Returns
///
/// * `hostname`: The value of the "DNS_HOSTNAME" environment variable.
/// * `token`: The value of the "TELEGRAM_TOKEN" environment variable.
fn verify_env_vars() -> (String, String) {
    // Define the environment variables to check
    let envvars: Vec<&str> = vec![
        "TELEGRAM_TOKEN",
        "DNS_HOSTNAME",
        "API_KEY",
        "API_SECRET",
        "URL",
        "CHAT_ID",
        "INTERFACE",
    ];

    // Check if the environment variables are set
    let error: bool = get_vars_from_env(envvars);
    if error {
        log::error!("One or more environment variables are missing");
        std::process::exit(1);
    }

    // Retrieve the values of "TELEGRAM_TOKEN" and "DNS_HOSTNAME"
    let token: String =
        get_var_from_env("TELEGRAM_TOKEN").unwrap_or_else(|_| std::process::exit(1));
    let hostname: String =
        get_var_from_env("DNS_HOSTNAME").unwrap_or_else(|_| std::process::exit(1));

    // Return the values
    (hostname, token)
}

/// Verifies the IP addresses associated with a given hostname and a token.
///
/// This function first resolves the hostname to an IP address using the `dns::resolve_hostname` function.
/// It then retrieves the WAN IP address using the `api::get_api` function.
///
/// If either the resolved IP address or the WAN IP address is empty (checked using the `is_empty` method),
/// it logs a warning and skips the comparison.
///
/// If both IP addresses are not empty and they don't match (checked using the `!=` operator),
/// it logs that the IP address is different. If the token is not empty,
/// it attempts to send a Telegram message with the `telegram::send_telegram` function.
///
/// If the IP addresses match, it attempts to send a successful update Telegram message.
///
/// The function then sleeps for 10 seconds using the `thread::sleep` function before incrementing a counter.
///
/// If the counter reaches 180 (indicating 30 minutes have passed), it resets the counter to 1 and logs that 30 minutes have passed.
///
/// # Arguments
///
/// * `hostname` - A string slice that holds the hostname.
/// * `token` - A string slice that holds the token.
/// * `counter` - A 32-bit integer that holds the counter.
///
/// # Returns
///
/// * A 32-bit integer that holds the updated counter.
fn verify_ips(hostname: &String, token: &String, counter: i32) -> i32 {
    // Log that IPs are being verified if counter is 0
    if counter == 0 {
        log::info!("Verifying IPs");
    }

    // Resolve the hostname to an IP address
    let ip_address = dns::resolve_hostname(hostname);
    if ip_address.is_empty() {
        log::warn!("Failed to get IP address");
    }

    // Retrieve the WAN IP address
    let wan_ip = api::get_api();
    if wan_ip.is_empty() {
        log::warn!("Failed to get WAN IP address");
    }

    // Log the IP addresses
    log::debug!(
        "The IP address of {} is: {}, WAN IP address is: {}",
        hostname,
        ip_address,
        wan_ip
    );

    // Compare the IP addresses
    if ip_address.is_empty() || wan_ip.is_empty() {
        log::warn!("Since one of the IP addresses is empty, skipping comparison");
    } else if ip_address != wan_ip {
        log::info!("IP address is different");
        if !token.is_empty() && !telegram::send_telegram(token, &ip_address, &wan_ip) {
            log::warn!("Failed to send telegram");
        } else {
            log::info!("Telegram sent");
        }
    } else if !telegram::send_telegram(token, &ip_address, &wan_ip) {
        log::warn!("Failed to send successful update telegram");
    }

    // Sleep for 10 seconds
    log::debug!("Sleeping for 10 seconds");
    sleep(Duration::new(1, 0));

    // Increment the counter
    let counter: i32 = counter + 1;
    if counter >= 180 {
        log::info!("30 minutes passed");
        return 1;
    }

    // Return the updated counter
    counter
}

/// Initializes the logging for the application.
///
/// This function first checks if the "RUST_LOG" environment variable is set using the `std::env::var` function.
/// If the "RUST_LOG" environment variable is not set (indicated by `std::env::var` returning an `Err`),
/// it sets the "RUST_LOG" environment variable to "INFO" using the `std::env::set_var` function.
///
/// It then initializes the logger with the environment variables using the `simple_logger::init_with_env` function.
/// If `simple_logger::init_with_env` fails, it will panic and terminate the program.
///
/// Finally, it logs that the DNS checker is starting using the `log::info` function.
fn init() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    simple_logger::init_with_env().unwrap();
    log::info!("Starting DNS checker");
}

/// Module for testing the functions in the parent module.
#[cfg(test)]
mod tests {
    // Import everything from the parent module
    use super::*;

    /// Tests the `init` function.
    ///
    /// This function calls `init` and then checks that the "RUST_LOG" environment variable is set to "INFO".
    /// If the "RUST_LOG" environment variable is not set to "INFO", the test will fail.
    #[test]
    fn test_init() {
        // Call the function
        init();

        // Assert that the function sets the "RUST_LOG" environment variable to "INFO"
        assert_eq!(std::env::var("RUST_LOG").unwrap(), "INFO");
    }
}
