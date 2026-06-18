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

**MX (Mail Exchange) records** specify mail servers for a domain. Each MX record has a priority value — lower values are preferred.

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

## TTL (Time to Live)

TTL tells DNS resolvers how long to cache a record. Lower TTLs (60–300 seconds) are useful when testing changes. Higher TTLs (3600–86400 seconds) reduce DNS query load.

## Best Practices

- Set low TTLs (60s) before making major changes
- Always verify records with `dig` or `nslookup`
- Keep at least two nameservers for redundancy
- Use CNAME records for subdomain delegation when possible
