export function greet(name: string): string {
  return `Hello, ${name}! Welcome to Bnpm.`
}

export function resolvePackage(name: string, version = "latest"): string {
  return `https://registry.bnpm.dev/${name}/-/${name}-${version}.tgz`
}

export const VERSION = "0.0.1"

export const BNPM_SDK_VERSION = "0.0.1"
