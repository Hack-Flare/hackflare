<div align="center">
</div>

# HackFlare
Cloudflare alternative for Hack Club. Hence the name HackFlare.  

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