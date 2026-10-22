export function sharedUtil(): string {
  return "shared utility from monorepo"
}

export function getPackageUrl(name: string, version = "latest"): string {
  return `https://registry.bnpm.dev/${name}/-/${name}-${version}.tgz`
}
