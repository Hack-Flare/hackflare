# Coolify Docker Image Deployment

The application can be deployed as a Coolify **Docker Image** resource.
Do not publish host port 53 from the resource.
Instead Coolify must be able to start the replacement container while the previous container is still running.

## Proxy entrypoints

Add these static entrypoints to the Coolify server proxy configuration under **Servers > Proxy > Configuration**:

```yaml
entryPoints:
  dns-tcp:
    address: ":53"
  dns-udp:
    address: ":53/udp"
```

Restart or reload the Coolify proxy after changing its static configuration.  
Traefik then owns host port 53 for the lifetime of the server, independently of application deployments.

## Application settings

Configure the Docker Image resource with:

- Image: `ghcr.io/hack-flare/hackflare-app`
- HTTP port: `8080`
- Additional exposed ports: `5353/tcp` and `5353/udp`
- Host port mappings: none
- Rolling updates: enabled
- A health check for `http://127.0.0.1:8080/health`

Configure the normal application domain through Coolify's Domains page.
The Docker image's runtime environment must keep `DNS_BIND_ADDR=0.0.0.0:5353`.

## Active labels

Add these labels in the Docker Image resource's **Container labels** section:

```text
traefik.enable=true
traefik.tcp.routers.hackflare-dns-tcp.entrypoints=dns-tcp
traefik.tcp.routers.hackflare-dns-tcp.rule=HostSNI(`*`)
traefik.tcp.routers.hackflare-dns-tcp.service=hackflare-dns-tcp
traefik.tcp.services.hackflare-dns-tcp.loadbalancer.server.port=5353
traefik.udp.routers.hackflare-dns-udp.entrypoints=dns-udp
traefik.udp.routers.hackflare-dns-udp.service=hackflare-dns-udp
traefik.udp.services.hackflare-dns-udp.loadbalancer.server.port=5353
```

Coolify connects its proxy to the application network.  
If the resource uses a custom network, add `traefik.docker.network=<proxy-connected-network>` with the actual Docker network name.

## Migration order

1. Stop the old Compose application and its direct `53:5353` mappings.
2. Add the `dns-tcp` and `dns-udp` entrypoints to the Coolify proxy.
3. Confirm the proxy owns ports 53 TCP and UDP.
4. Create the Docker Image resource with no host port mappings.
5. Add the labels and wait for the HTTP health check to pass.
6. Verify both TCP and UDP DNS queries through the public nameserver.
7. Enable automatic deployments and rolling updates.
8. Remove the old Compose resource after the new resource is verified.

The initial migration has a one-time handoff because the old application owns port 53 directly.  
Subsequent Docker Image deployments can overlap old and new containers because Traefik remains the sole owner of port 53.
