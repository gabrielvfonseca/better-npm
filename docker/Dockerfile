# syntax=docker.io/docker/dockerfile:1.7

FROM oven/bun:1.2-alpine AS base
WORKDIR /app

ARG APP
ARG PACKAGE=${APP}

# Stage 1: Prune with turbo
FROM base AS pruner
RUN bun install -g turbo
COPY package.json bun.lock turbo.json ./
COPY . .
ARG APP
ARG PACKAGE
RUN turbo prune --docker @better-npm/${PACKAGE}

# Stage 2: Install dependencies
FROM base AS deps
COPY --from=pruner /app/out/json/ .
COPY --from=pruner /app/out/bun.lock ./bun.lock
RUN --mount=type=cache,target=/root/.bun bun install --frozen-lockfile

# Stage 3: Build
FROM base AS builder
COPY --from=pruner /app/out/full/ .
COPY --from=deps /app/node_modules ./node_modules
ARG APP
ARG PACKAGE
RUN --mount=type=cache,target=/root/.turbo bun run build --filter=@better-npm/${PACKAGE}

# Stage 4: Runner
FROM base AS runner
ARG APP
ARG PORT=3000
RUN addgroup -S bnpm && adduser -S -G bnpm bnpm
COPY --from=builder --chown=bnpm:bnpm /app/apps/${APP}/dist ./dist
COPY --from=builder --chown=bnpm:bnpm /app/apps/${APP}/package.json ./
COPY --from=builder --chown=bnpm:bnpm /app/node_modules ./node_modules
USER bnpm
ENV NODE_ENV=production
ENV PORT=${PORT}
EXPOSE ${PORT}
CMD ["bun", "dist/index.js"]
