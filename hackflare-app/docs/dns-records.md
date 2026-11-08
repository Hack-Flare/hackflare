# DNS Records

A guide to the different types of DNS records and how to use them effectively.

## A Records

**A (Address) records** map a domain name to an IPv4 address. This is the most common record type for pointing a domain to a web server.

```
example.com.  A  198.51.100.1
```

## AAAA Records

**AAAA records** are like A records but for IPv6 addresses. If your server supports IPv6, you should create both A and AAAA records.

```
example.com.  AAAA  2001:db8::1
```

## CNAME Records

**CNAME (Canonical Name) records** alias one domain name to another. They're commonly used to point `www.example.com` to `example.com`.

```
www.example.com.  CNAME  example.com.
```

> Note: CNAME records cannot coexist with other record types at the same name.

## MX Records

**MX (Mail Exchange) records** specify mail servers for a domain. Each MX record has a priority value - lower values are preferred.

```
example.com.  MX  10  mail.example.com.
example.com.  MX  20  backup-mail.example.com.
```

## TXT Records

**TXT records** hold text information. They're commonly used for:

- SPF records (email authentication)
- DKIM keys (email signing)
- Domain verification tokens
- DMARC policies

```
example.com.  TXT  "v=spf1 include:_spf.google.com ~all"
```

## ANAME Records

**ANAME records** alias the zone apex to another hostname without using a CNAME at
the apex. This is useful when the target is a CDN or hosted service.

```
example.com.  ANAME  app.example.net.
```

## NS Records

**NS records** delegate a name to a set of authoritative nameservers. The apex NS
records are managed by Hackflare.

```
sub.example.com.  NS  ns1.example.net.
sub.example.com.  NS  ns2.example.net.
```

## SRV Records

**SRV records** publish the host and port for a service. The fields are priority,
weight, port, and target.

```
_sip._tcp.example.com.  SRV  10 5 5060 sip.example.com.
```

## CAA Records

**CAA records** restrict which certificate authorities may issue certificates for
a name.

```
example.com.  CAA  0 issue "letsencrypt.org"
```

## DNSSEC Records

**DS**, **DNSKEY**, **RRSIG**, and **CDS** records publish DNSSEC keys, delegation
signing information, and signatures.

```
example.com.  DNSKEY  257 3 13 mdsswUyr3DPW132mOi8V9xESWE8jTo0dxCjjnopKl+GqJxpVXckHAeF+KkxLbxILfDLUT0rAK9iUzy1L53eKGQ==
example.com.  DS  12345 13 2 49FD46E6C4B45C55D4AC...
```

## Service Binding Records

**SVCB** and **HTTPS** records describe alternative endpoints and connection
parameters. Parameters are written as `key=value` pairs.

```
example.com.  HTTPS  1 . alpn=h2,h3 port=443 ipv4hint=192.0.2.1
```

## Other Record Types

Every IANA record type can be stored, including types without a dedicated parser.
Use the RFC 3597 generic presentation format for those:

```
name.example.com.  LOC  \# 16 0012160000000000...
```

The format is `\# <length> <hex>`, where `<length>` is the byte length of the
RDATA and `<hex>` is its wire encoding.

## TTL (Time to Live)

TTL tells DNS resolvers how long to cache a record. Lower TTLs (60–300 seconds) are useful when testing changes. Higher TTLs (3600–86400 seconds) reduce DNS query load.

## Best Practices

- Set low TTLs (60s) before making major changes
- Always verify records with `dig` or `nslookup`
- Keep at least two nameservers for redundancy
- Use CNAME records for subdomain delegation when possible
