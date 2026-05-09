FROM oven/bun:latest

WORKDIR /app

COPY Frontend/package.json Frontend/bun.lockb ./
RUN bun install --frozen-lockfile || bun install

COPY Frontend ./

EXPOSE 3000

CMD ["bun", "run", "dev", "--host", "0.0.0.0", "--port", "3000"]