export function greet(name: string): string {
  return `Hello, ${name}! Welcome to Dash.`;
}

export function resolvePackage(name: string, version = 'latest'): string {
  return `https://registry.dash.dev/${name}/-/${name}-${version}.tgz`;
}

export const VERSION = '0.0.1';

export const DASH_SDK_VERSION = '0.0.1';
