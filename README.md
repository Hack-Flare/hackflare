<div align="center">
<!-- Logo goes here in future -->
</div>

# HackFlare

Cloudflare alternative for Hack Club. Hence the name HackFlare.  

## Introduction

HackFlare is a project that aims to provide a free, open-source and open-infra alternative to Cloudflare.
It is designed to be easy to use, easy to deploy and realtively easy to maintain.  

### Features

Note: These are the active features. For planned features please see the [roadmap](docs/ROADMAP.md).

**Frontend:**

- Dashboard
- Domain management
- Admin panel

**Backend:**

- DNS management
- Account Registration and Authentication
  - [HackClub Auth](https://auth.hackclub.com/docs/welcome)
  - [GitHub OAuth](https://docs.github.com/en/developers/apps/building-oauth-apps/authorizing-oauth-apps)
  - [Google OAuth](https://developers.google.com/identity/protocols/oauth2)
  - Email and Password Auth
- Account management

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