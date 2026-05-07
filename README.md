<div align="center">
<img alt="upscalemedia-transformed (1)" src="https://github.com/user-attachments/assets/d99965b5-523d-4878-b985-93a841477db2" />
</div>

# HackFlare
Cloudflare alternative for HackClub. Hence the name HackFlare.  
[View Elixir Documentation](ELIXIR_DOCS.md)

## Stack

PETRL  
- Phoenix
- Elixir
- Tailwind
- Rust
- liveview

## Running the project

Without Docker
```
iex -S mix phx.server
```

With Docker from prebuilt image
```
docker compose up
```

Dev Docker Which builds the image locally
```
docker compose -f docker-compose.dev.yml up
```
