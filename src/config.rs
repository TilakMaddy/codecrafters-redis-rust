use std::net::Ipv4Addr;

struct IPConfig {
    ip_address: Ipv4Addr,
    port: u16
}

impl From<IPConfig> for String {
    fn from(value: IPConfig) -> Self {
        format!("{}:{}", value.ip_address.to_string(), value.port)
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use crate::config::IPConfig;

    #[test]
    fn config_to_string() {
        let ip = IPConfig {
            ip_address: Ipv4Addr::new(127, 0, 0, 1),
            port: 6379
        };
        let ip_str: String = ip.into();
        assert_eq!(ip_str, "127.0.0.1:6379");
    }

}