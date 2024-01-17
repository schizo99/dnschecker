use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::Resolver; // Import the ResolverConfig and ResolverOpts structs

/// Resolves a hostname to its corresponding IPv4 address.
///
/// This function takes a hostname as an argument.
/// It creates a `ResolverConfig` using the `ResolverConfig::google` function, which uses Google's DNS resolver.
/// It also creates default `ResolverOpts` using the `ResolverOpts::default` function.
///
/// It then creates a `Resolver` using the `Resolver::new` function with the `ResolverConfig` and `ResolverOpts`.
/// If the function fails, it logs a warning and returns an empty `String`.
///
/// It then attempts to look up the IP address of the hostname using the `Resolver::lookup_ip` function.
/// If the function fails, it logs a warning and returns an empty `String`.
///
/// It then iterates over the returned IP addresses and finds the first IPv4 address.
/// If no IPv4 address is found, it logs a warning and returns an empty `String`.
/// If an IPv4 address is found, it returns its value as a `String`.
///
/// # Arguments
///
/// * `hostname`: A `&str` that specifies the hostname to resolve.
///
/// # Returns
///
/// * A `String` that holds the IPv4 address of the hostname if the function succeeds.
/// * If any step fails, it returns an empty `String`.
pub fn resolve_hostname(hostname: &str) -> String {
    let resolver = match Resolver::new(ResolverConfig::google(), ResolverOpts::default()) {
        Ok(resolver) => resolver,
        Err(err) => {
            log::warn!("Failed to build resolver: {}", err);
            return String::new();
        }
    };

    match resolver.lookup_ip(hostname) {
        Ok(response) => {
            let ipv4_address = response
                .iter()
                .find(|ip| ip.is_ipv4())
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| {
                    log::warn!("No IPv4 addresses found for hostname: {}", hostname);
                    String::new()
                });
            ipv4_address
        }
        Err(err) => {
            log::warn!(
                "Failed to lookup IP address: {} for hostname: {}",
                err,
                hostname
            );
            String::new()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_hostname() {
        let result = resolve_hostname("localhost");

        assert_eq!(result, "127.0.0.1");
    }
}
