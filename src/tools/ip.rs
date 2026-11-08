use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::error::{Result, ToolError};
use crate::tools::number::format_thousands;

/// Parses an IPv4/IPv6 address or CIDR block and describes the network.
pub fn inspect(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ToolError::new("No IP address provided.")
            .with_hint("Try `192.168.1.42/24`, `10.0.0.1` or `2001:db8::1/64`."));
    }

    let (address_text, prefix_text) = match trimmed.split_once('/') {
        Some((address, prefix)) => (address, Some(prefix)),
        None => (trimmed, None),
    };

    let address: IpAddr = address_text.parse().map_err(|_| {
        ToolError::new(format!("`{address_text}` is not a valid IP address."))
            .with_hint("IPv4 looks like `192.168.1.1`; IPv6 looks like `2001:db8::1`.")
    })?;

    match address {
        IpAddr::V4(ip) => {
            let prefix = parse_prefix(prefix_text, 32, "IPv4")?;
            Ok(describe_v4(ip, prefix))
        }
        IpAddr::V6(ip) => {
            let prefix = parse_prefix(prefix_text, 128, "IPv6")?;
            Ok(describe_v6(ip, prefix))
        }
    }
}

fn parse_prefix(value: Option<&str>, maximum: u8, family: &str) -> Result<u8> {
    let Some(value) = value else {
        return Ok(maximum);
    };
    let prefix: u8 = value.trim().parse().map_err(|_| {
        ToolError::new(format!("`{value}` is not a valid {family} prefix length."))
            .with_hint(format!("Prefix lengths range from 0 to {maximum}."))
    })?;
    if prefix > maximum {
        return Err(ToolError::new(format!(
            "{family} prefix length must be between 0 and {maximum}."
        )));
    }
    Ok(prefix)
}

fn describe_v4(ip: Ipv4Addr, prefix: u8) -> String {
    let raw = u32::from(ip);
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - u32::from(prefix))
    };
    let network = raw & mask;
    let broadcast = network | !mask;
    let total = 1u64 << (32 - u32::from(prefix));
    let usable = match prefix {
        32 => 1,
        31 => 2,
        _ => total.saturating_sub(2),
    };
    let (first, last) = match prefix {
        32 => (network, broadcast),
        31 => (network, broadcast),
        _ => (network.saturating_add(1), broadcast.saturating_sub(1)),
    };
    let assumed = if prefix == 32 { " (assumed host)" } else { "" };

    [
        format!("Address    : {ip}"),
        "Version    : IPv4".to_owned(),
        format!("Prefix     : /{prefix}{assumed}"),
        format!("Network    : {}", Ipv4Addr::from(network)),
        format!("Broadcast  : {}", Ipv4Addr::from(broadcast)),
        format!("First host : {}", Ipv4Addr::from(first)),
        format!("Last host  : {}", Ipv4Addr::from(last)),
        format!("Netmask    : {}", Ipv4Addr::from(mask)),
        format!("Wildcard   : {}", Ipv4Addr::from(!mask)),
        format!("Addresses  : {}", format_thousands(u128::from(total))),
        format!("Usable     : {}", format_thousands(u128::from(usable))),
        format!("Scope      : {}", scope_v4(ip)),
    ]
    .join("\n")
}

fn describe_v6(ip: Ipv6Addr, prefix: u8) -> String {
    let raw = u128::from(ip);
    let mask = if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - u32::from(prefix))
    };
    let network = raw & mask;
    let last = network | !mask;
    let host_bits = 128 - u32::from(prefix);
    let total = if host_bits >= 128 {
        String::from("2^128")
    } else {
        (1u128 << host_bits).to_string()
    };
    let expanded = ip
        .segments()
        .iter()
        .map(|segment| format!("{segment:04x}"))
        .collect::<Vec<_>>()
        .join(":");
    let assumed = if prefix == 128 { " (assumed host)" } else { "" };

    [
        format!("Address    : {ip}"),
        "Version    : IPv6".to_owned(),
        format!("Expanded   : {expanded}"),
        format!("Prefix     : /{prefix}{assumed}"),
        format!("Network    : {}", Ipv6Addr::from(network)),
        format!("First addr : {}", Ipv6Addr::from(network)),
        format!("Last addr  : {}", Ipv6Addr::from(last)),
        format!("Netmask    : {}", Ipv6Addr::from(mask)),
        format!("Wildcard   : {}", Ipv6Addr::from(!mask)),
        format!("Addresses  : {total}"),
        format!("Scope      : {}", scope_v6(ip)),
    ]
    .join("\n")
}

fn scope_v4(ip: Ipv4Addr) -> &'static str {
    if ip.is_unspecified() {
        "unspecified"
    } else if ip.is_loopback() {
        "loopback"
    } else if ip.is_private() {
        "private"
    } else if ip.is_link_local() {
        "link-local"
    } else if ip.is_broadcast() {
        "broadcast"
    } else if ip.is_multicast() {
        "multicast"
    } else if ip.is_documentation() {
        "documentation"
    } else {
        "public"
    }
}

fn scope_v6(ip: Ipv6Addr) -> &'static str {
    if ip.is_unspecified() {
        return "unspecified";
    }
    if ip.is_loopback() {
        return "loopback";
    }
    if ip.is_multicast() {
        return "multicast";
    }
    if ip.to_ipv4_mapped().is_some() {
        return "IPv4-mapped";
    }
    let segments = ip.segments();
    if segments[0] & 0xfe00 == 0xfc00 {
        return "unique-local";
    }
    if segments[0] & 0xffc0 == 0xfe80 {
        return "link-local";
    }
    if segments[0] == 0x2001 && segments[1] == 0x0db8 {
        return "documentation";
    }
    "global"
}

#[cfg(test)]
mod tests {
    use super::inspect;

    #[test]
    fn describes_ipv4_subnets() -> Result<(), Box<dyn std::error::Error>> {
        let output = inspect("192.168.1.42/24")?;
        assert!(output.contains("Network    : 192.168.1.0"));
        assert!(output.contains("Broadcast  : 192.168.1.255"));
        assert!(output.contains("Usable     : 254"));
        assert!(output.contains("Scope      : private"));

        let output = inspect("10.0.0.0/31")?;
        assert!(output.contains("Usable     : 2"));
        Ok(())
    }

    #[test]
    fn describes_ipv6_subnets() -> Result<(), Box<dyn std::error::Error>> {
        let output = inspect("2001:db8::1/64")?;
        assert!(output.contains("Network    : 2001:db8::"));
        assert!(output.contains("Addresses  : 18446744073709551616"));
        assert!(output.contains("Scope      : documentation"));
        Ok(())
    }

    #[test]
    fn validates_addresses_and_prefixes() {
        assert!(inspect("192.168.1.1/33").is_err());
        assert!(inspect("2001:db8::1/129").is_err());
        assert!(inspect("not-an-ip").is_err());
    }
}
