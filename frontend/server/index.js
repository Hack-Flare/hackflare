import { createProxyMiddleware } from "http-proxy-middleware"
import compression from "compression"
import express from "express"
import morgan from "morgan"
import path from "node:path"
import url from "node:url"
import { createRequestHandler } from "@react-router/express"

const PORT = process.env.PORT || 3000
const BUILD_PATH = path.resolve("build/server/index.js")

const build = await import(url.pathToFileURL(BUILD_PATH).href)

const app = express()

app.disable("x-powered-by")
app.use(compression())

app.get("/health", (_req, res) => res.sendStatus(200))

const API_PROXY_TARGET = process.env.API_PROXY_TARGET || "http://localhost:8080"
app.use(
  "/api",
  createProxyMiddleware({
    target: API_PROXY_TARGET,
    changeOrigin: true,
  })
)

app.use(
  path.posix.join("/", "assets"),
  express.static(path.resolve("build/client/assets"), {
    immutable: true,
    maxAge: "1y",
  })
)
app.use("/", express.static(path.resolve("build/client")))
app.use("/", express.static("public", { maxAge: "1h" }))
app.use(morgan("tiny"))

app.all(
  "*",
  createRequestHandler({
    build,
    mode: process.env.NODE_ENV,
  })
)

const server = app.listen(PORT, () => {
  console.log(`[server] http://localhost:${PORT}`)
})

;["SIGTERM", "SIGINT"].forEach((signal) => {
  process.once(signal, () => server.close(console.error))
})
