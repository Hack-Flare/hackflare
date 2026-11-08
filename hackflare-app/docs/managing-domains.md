# Managing Domains

Once your domain is verified, you can manage it from the dashboard.

## Domain Overview

The Domains page shows all your domains with their current status, including:

- **Active**, Domain is verified and DNS is managed by HackFlare
- **Pending**, Domain is awaiting verification
- **Error**, There was an issue with verification or DNS resolution

## DNS Management

Click on a domain name to access its DNS management page. Here you can:

- View all DNS records
- Add new records
- Edit existing records
- Delete records

## Supported Record Types

Every record type in the IANA DNS registry can be stored and served. Common types
have dedicated parsers and validation:

| Type | Purpose |
|------|---------|
| A | Maps a domain to an IPv4 address |
| AAAA | Maps a domain to an IPv6 address |
| ANAME | Alias at the zone apex |
| CAA | Certificate Authority Authorization |
| CNAME | Maps a domain to another domain name |
| CERT | Certificate records |
| DNAME | Redirects an entire subtree |
| DNSKEY | DNSSEC public keys |
| DS | DNSSEC delegation signer records |
| EUI48 | 48-bit hardware addresses |
| EUI64 | 64-bit hardware addresses |
| HTTPS | HTTPS service binding |
| HINFO | Host CPU and operating system |
| KX | Key exchanger records |
| MX | Mail exchange records for email routing |
| NAPTR | Naming Authority Pointer records |
| NS | Nameserver records |
| PTR | Reverse DNS pointers |
| RP | Responsible person records |
| RRSIG | DNSSEC signatures |
| SOA | Zone authority records |
| SPF | Sender Policy Framework records |
| SRV | Service records |
| SSHFP | SSH host key fingerprints |
| SVCB | Service binding records |
| TLSA | TLS certificate association records |
| TXT | Text records for verification and SPF/DKIM |
| URI | Service URI records |

Types without a dedicated parser can still be managed with the RFC 3597 generic
format: `\# <length> <hex>`. For example, a LOC record uses
`\# 16 0012160000000000...` following its wire encoding.

## Bulk Operations

You can select multiple records and perform bulk actions:

- **Delete**, Remove selected records
- **Export**, Download records as a zone file

## Redirects

The Redirects section lets you set up URL forwarding rules for your domain.
