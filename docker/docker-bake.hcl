variable "REGISTRY" {
  default = "ghcr.io/bnpm-io"
}

variable "TAG" {
  default = "latest"
}

variable "CACHE_FROM" {
  default = "type=gha"
}

variable "CACHE_TO" {
  default = "type=gha,mode=max"
}

target "api" {
  context    = ".."
  dockerfile = "./Dockerfile"
  tags       = ["${REGISTRY}/bnpm-api:${TAG}"]
  args = {
    APP     = "api"
    PACKAGE = "app-api"
    PORT    = "3000"
  }
  cache-from = ["${CACHE_FROM}"]
  cache-to   = ["${CACHE_TO}"]
}

target "registry" {
  context    = ".."
  dockerfile = "./Dockerfile"
  tags       = ["${REGISTRY}/bnpm-registry:${TAG}"]
  args = {
    APP     = "registry"
    PACKAGE = "registry"
    PORT    = "4000"
  }
  cache-from = ["${CACHE_FROM}"]
  cache-to   = ["${CACHE_TO}"]
}

target "worker-downloads" {
  context    = ".."
  dockerfile = "./Dockerfile.worker"
  tags       = ["${REGISTRY}/bnpm-worker-downloads:${TAG}"]
  args = {
    WORKER_NAME = "downloads"
  }
  cache-from = ["${CACHE_FROM}"]
  cache-to   = ["${CACHE_TO}"]
}

target "worker-indexing" {
  context    = ".."
  dockerfile = "./Dockerfile.worker"
  tags       = ["${REGISTRY}/bnpm-worker-indexing:${TAG}"]
  args = {
    WORKER_NAME = "indexing"
  }
  cache-from = ["${CACHE_FROM}"]
  cache-to   = ["${CACHE_TO}"]
}

target "worker-metadata" {
  context    = ".."
  dockerfile = "./Dockerfile.worker"
  tags       = ["${REGISTRY}/bnpm-worker-metadata:${TAG}"]
  args = {
    WORKER_NAME = "metadata"
  }
  cache-from = ["${CACHE_FROM}"]
  cache-to   = ["${CACHE_TO}"]
}

target "worker-scanning" {
  context    = ".."
  dockerfile = "./Dockerfile.worker"
  tags       = ["${REGISTRY}/bnpm-worker-scanning:${TAG}"]
  args = {
    WORKER_NAME = "scanning"
  }
  cache-from = ["${CACHE_FROM}"]
  cache-to   = ["${CACHE_TO}"]
}

target "worker-signatures" {
  context    = ".."
  dockerfile = "./Dockerfile.worker"
  tags       = ["${REGISTRY}/bnpm-worker-signatures:${TAG}"]
  args = {
    WORKER_NAME = "signatures"
  }
  cache-from = ["${CACHE_FROM}"]
  cache-to   = ["${CACHE_TO}"]
}

target "worker-analytics" {
  context    = ".."
  dockerfile = "./Dockerfile.worker"
  tags       = ["${REGISTRY}/bnpm-worker-analytics:${TAG}"]
  args = {
    WORKER_NAME = "analytics"
  }
  cache-from = ["${CACHE_FROM}"]
  cache-to   = ["${CACHE_TO}"]
}

target "worker-cleanup" {
  context    = ".."
  dockerfile = "./Dockerfile.worker"
  tags       = ["${REGISTRY}/bnpm-worker-cleanup:${TAG}"]
  args = {
    WORKER_NAME = "cleanup"
  }
  cache-from = ["${CACHE_FROM}"]
  cache-to   = ["${CACHE_TO}"]
}

group "all" {
  targets = [
    "api",
    "registry",
    "worker-downloads",
    "worker-indexing",
    "worker-metadata",
    "worker-scanning",
    "worker-signatures",
    "worker-analytics",
    "worker-cleanup",
  ]
}

group "apps" {
  targets = ["api", "registry"]
}

group "workers" {
  targets = [
    "worker-downloads",
    "worker-indexing",
    "worker-metadata",
    "worker-scanning",
    "worker-signatures",
    "worker-analytics",
    "worker-cleanup",
  ]
}
