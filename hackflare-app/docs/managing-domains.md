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

| Type | Purpose |
|------|---------|
| A | Maps a domain to an IPv4 address |
| AAAA | Maps a domain to an IPv6 address |
| CNAME | Maps a domain to another domain name |
| MX | Mail exchange records for email routing |
| TXT | Text records for verification and SPF/DKIM |
| NS | Nameserver records |
| SRV | Service records |
| CAA | Certificate Authority Authorization |

## Bulk Operations

You can select multiple records and perform bulk actions:

- **Delete**, Remove selected records
- **Export**, Download records as a zone file

## Redirects

The Redirects section lets you set up URL forwarding rules for your domain.
