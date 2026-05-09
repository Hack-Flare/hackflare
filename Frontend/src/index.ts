import { serve } from "bun";
import index from "./index.html";

const backendBase = process.env.BACKEND_API_BASE ?? "http://127.0.0.1:8080";

async function proxyToBackend(req: Request): Promise<Response> {
  const incoming = new URL(req.url);
  const upstreamUrl = new URL(incoming.pathname + incoming.search, backendBase);

  const headers = new Headers(req.headers);
  headers.delete("host");

  const method = req.method.toUpperCase();
  const body = method === "GET" || method === "HEAD" ? undefined : await req.arrayBuffer();

  const upstream = await fetch(upstreamUrl, {
    method,
    headers,
    body,
    redirect: "manual",
  });

  return new Response(upstream.body, {
    status: upstream.status,
    statusText: upstream.statusText,
    headers: upstream.headers,
  });
}

const server = serve({
  routes: {
    "/api/v1/*": proxyToBackend,
    "/health": proxyToBackend,

    "/*": index,
  },

  development: process.env.NODE_ENV !== "production" && {
    hmr: true,
    console: true,
  },
});

console.log(`🚀 Server running at ${server.url}`);
