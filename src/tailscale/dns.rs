use crate::tailscale::utils::{sanitize_hostname, trim_suffix, Machine, PeerKind};

pub fn dns_or_quote_hostname(m: &mut Machine, dns_suffix: &str) {
    let base_name = trim_suffix(&m.dns_name, dns_suffix);
    m.display_name = match base_name {
        n if n.is_empty() => PeerKind::DNSName(sanitize_hostname(m.hostname.as_str())),
        base => PeerKind::HostName(base),
    }
}
