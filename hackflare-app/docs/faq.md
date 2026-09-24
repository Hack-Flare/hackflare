# Frequently Asked Questions

## General

### What is HackFlare?

HackFlare is a DNS management platform built for developers and organizations. It provides fast, reliable DNS hosting with a simple dashboard and REST API. (GRPC API SOON!!)

### Is HackFlare free?

HackFlare is free for anyone! Paid plans will be made in future to support hosting.
Check the dashboard for any usage limits.

### How is HackFlare different from Cloudflare?

Hackflare doesn't go down due to a once in a life time outage ever few months. Hackflare is open source and open infra, you can host it yourself instead of relying on a third party.

## Technical

### How long does DNS propagation take?

DNS propagation typically takes a few minutes but can take up to 48 hours depending on your domain registrar and TTL settings.

### Can I use my own nameservers?

Yes, you can use custom nameservers. The setup instructions are provided when you add a domain.

### Do you support DNSSEC?

DNSSEC support is planned for a future release.

### What TTL values do you recommend?

For testing, use 60 seconds. For production, we recommend 3600 seconds (1 hour) or higher.

## Account

### How do I reset my password?

Click "Forgot Password" on the login page and follow the instructions to reset your password.

### How do I delete my account?

Contact support through GitHub or Slack to request account deletion. We will add a self-service option soon.

### Can I have multiple API keys?

Yes, you can generate multiple API keys from the settings page. Each key can be labeled for different purposes.

## Troubleshooting

### My domain status shows "Pending" for a long time

This usually means the nameservers haven't been updated yet. Double-check your domain registrar settings and ensure you've copied the nameservers correctly.

Additionally click the Verify button to retrigger nameserver checks.

### SSL certificate failed to issue

Check that port 80 is accessible on your server. Some issuers also require a valid A or AAAA record pointing to your server.

### Changes aren't showing up

DNS changes can take time to propagate. Use `dig` or `nslookup` to check if the changes have propagated to your DNS resolver.
