<div align="center">
<!-- Logo goes here in future -->
</div>

# HackFlare

Cloudflare alternative, for free and open-source.  
Made by HackClubbers from [Hack Club](https://hackclub.com/) as an alternative to Cloudflare. Hence the name HackFlare.

> [!IMPORTANT]  
> Hackflare is still in development, if production goes down, please let us know!
> Although Hackflare is production ready, we do not have the best server for it yet.

## Introduction

HackFlare is a project that aims to provide a free, open-source and open-infra alternative to Cloudflare.
It is designed to be easy to use, easy to deploy and realtively easy to maintain.  

### Features

Note: These are the active features. For planned features please see the [roadmap](docs/ROADMAP.md).

**Frontend:**

- Dashboard
- Domain management
- Admin panel
- Logging
- Analytics
- Settings

**Backend:**

- DNS management
- Account Registration and Authentication
  - [HackClub Auth](https://auth.hackclub.com/docs/welcome)
  - Email and Password Auth
- Account management
- Authoritative DNS server
- Recursive DNS server
- PostgreSQL persistence
- Working internal API

## Roadmap

See [ROADMAP.md](docs/ROADMAP.md) for the roadmap.

## Running the project

### Production

```
docker compose -f deployment/compose.prod.yml up
```

### Development

```
docker compose -f deployment/compose.dev.yml --profile frontend --profile backend up
```

## License

Hackflare is licensed under Apache 2.0. See [LICENSE.md](LICENSE.md) for more details.