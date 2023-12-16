use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts}; // Import the ResolverConfig and ResolverOpts structs


pub fn resolve_hostname(hostname: &str) -> String {
    let config = ResolverConfig::google(); // Create a ResolverConfig using the google() function
    let options = ResolverOpts::default(); // Create default ResolverOpts

    let resolver = match Resolver::new(config, options) {
        Ok(resolver) => resolver,
        Err(err) => {
            log::warn!("Failed to build resolver: {}", err);
            return String::new();
        }
    };

    match resolver.lookup_ip(hostname) {
        Ok(response) => {
            let ipv4_address = response.iter()
            .find(|ip| ip.is_ipv4())
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| {
                log::warn!("No IPv4 addresses found for hostname: {}", hostname);
                String::new()
            });
            return ipv4_address;},
        Err(err) => {
            log::warn!("Failed to lookup IP address: {}", err);
            return String::new();
        }
    };

}