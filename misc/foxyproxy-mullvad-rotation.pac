// Mullvad Daily SOCKS5 Rotation PAC
// Intended for FoxyProxy / Chrome via GitHub Raw.
//
// Behavior:
// - Local/private destinations bypass the proxy.
// - Public sites are deterministically assigned to one Mullvad SOCKS5 exit per local calendar day.
// - The same base site keeps the same proxy until local midnight.
// - Berlin + Duesseldorf Mullvad SOCKS5 servers are used.
// - No credentials or secrets are embedded here.

var PROXIES = [
    "SOCKS5 de-ber-wg-socks5-001.relays.mullvad.net:1080",
    "SOCKS5 de-ber-wg-socks5-002.relays.mullvad.net:1080",
    "SOCKS5 de-ber-wg-socks5-003.relays.mullvad.net:1080",
    "SOCKS5 de-ber-wg-socks5-004.relays.mullvad.net:1080",
    "SOCKS5 de-ber-wg-socks5-101.relays.mullvad.net:1080",
    "SOCKS5 de-ber-wg-socks5-102.relays.mullvad.net:1080",
    "SOCKS5 de-ber-wg-socks5-103.relays.mullvad.net:1080",
    "SOCKS5 de-dus-wg-socks5-001.relays.mullvad.net:1080",
    "SOCKS5 de-dus-wg-socks5-002.relays.mullvad.net:1080",
    "SOCKS5 de-dus-wg-socks5-003.relays.mullvad.net:1080",
    "SOCKS5 de-dus-wg-socks5-101.relays.mullvad.net:1080",
    "SOCKS5 de-dus-wg-socks5-102.relays.mullvad.net:1080",
    "SOCKS5 de-dus-wg-socks5-103.relays.mullvad.net:1080"
];

var TWO_LEVEL_SUFFIXES = {
    "co.uk":1, "org.uk":1, "me.uk":1, "ac.uk":1, "gov.uk":1,
    "com.au":1, "net.au":1, "org.au":1,
    "co.nz":1, "net.nz":1, "org.nz":1,
    "co.jp":1, "ne.jp":1, "or.jp":1,
    "com.br":1, "com.mx":1, "com.tr":1,
    "com.sg":1, "com.hk":1, "com.cn":1,
    "co.kr":1, "co.in":1
};

function isIPv4Literal(host) {
    return /^\d{1,3}(\.\d{1,3}){3}$/.test(host);
}

function isPrivateIPv4(ip) {
    var p = ip.split(".");
    if (p.length != 4) return false;

    var a = parseInt(p[0], 10);
    var b = parseInt(p[1], 10);

    if (a == 10) return true;
    if (a == 127) return true;
    if (a == 169 && b == 254) return true;
    if (a == 172 && b >= 16 && b <= 31) return true;
    if (a == 192 && b == 168) return true;

    return false;
}

function isLocalHost(host) {
    host = host.toLowerCase();

    if (host == "localhost" ||
        host == "::1" ||
        host == "[::1]" ||
        dnsDomainIs(host, ".local") ||
        isPlainHostName(host)) {
        return true;
    }

    if (isIPv4Literal(host) && isPrivateIPv4(host)) {
        return true;
    }

    return false;
}

function siteKey(host) {
    host = host.toLowerCase();

    if (host.charAt(host.length - 1) == ".") {
        host = host.substring(0, host.length - 1);
    }

    if (isIPv4Literal(host)) return host;

    var parts = host.split(".");
    if (parts.length <= 2) return host;

    var suffix2 = parts[parts.length - 2] + "." + parts[parts.length - 1];

    if (TWO_LEVEL_SUFFIXES[suffix2] && parts.length >= 3) {
        return parts[parts.length - 3] + "." + suffix2;
    }

    return suffix2;
}

function dayKey() {
    var d = new Date();
    var y = d.getFullYear();
    var m = d.getMonth() + 1;
    var day = d.getDate();

    return y + "-" + (m < 10 ? "0" : "") + m + "-" + (day < 10 ? "0" : "") + day;
}

function hashString(s) {
    var h = 2166136261;

    for (var i = 0; i < s.length; i++) {
        h ^= s.charCodeAt(i);
        h += (h << 1) + (h << 4) + (h << 7) + (h << 8) + (h << 24);
        h = h >>> 0;
    }

    return h >>> 0;
}

function FindProxyForURL(url, host) {
    host = host.toLowerCase();

    if (isLocalHost(host)) {
        return "DIRECT";
    }

    var resolved = dnsResolve(host);
    if (resolved && isIPv4Literal(resolved) && isPrivateIPv4(resolved)) {
        return "DIRECT";
    }

    var key = siteKey(host) + "|" + dayKey();
    var index = hashString(key) % PROXIES.length;

    return PROXIES[index];
}
