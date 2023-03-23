use std::net::Ipv4Addr;

pub struct IPConfig {
    pub ip_address: Ipv4Addr,
    pub port: u16
}

impl From<IPConfig> for String {
    fn from(value: IPConfig) -> Self {
        format!("{}:{}", value.ip_address, value.port)
    }
}

pub fn redis_defaults() -> String {
    let ip = IPConfig {
        ip_address: Ipv4Addr::new(127, 0, 0, 1),
        port: 6379
    };
    let ip_str: String = ip.into();
    return ip_str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_to_string() {
        let ip_str = redis_defaults();
        assert_eq!(ip_str, "127.0.0.1:6379");
    }

}