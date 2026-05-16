<div align="center">
<img alt="upscalemedia-transformed (1)" src="https://github.com/user-attachments/assets/d99965b5-523d-4878-b985-93a841477db2" />
</div>

# HackFlare
Cloudflare alternative for Hack Club. Hence the name HackFlare.  

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the roadmap.

## Running the project

With Docker from prebuilt image
```
docker compose up
```

Dev Backend Docker
```
docker compose -f compose.dev.yml --profile backend up
```
Dev Frontend Docker
```
docker compose -f compose.dev.yml --profile frontend up
```
Dev All in One Docker
```
docker compose -f compose.dev.yml --profile backend --profile frontend up
```

## License

Hackflare is licensed under Apache 2.0. See [LICENSE.md](LICENSE.md) for more details.