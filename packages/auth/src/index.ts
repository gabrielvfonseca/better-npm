export interface AuthToken {
  sub: string
  email: string
  roles: string[]
  iat: number
  exp: number
  jti: string
}

export type Result<T, E = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E }

function base64UrlEncode(data: string): string {
  return btoa(data)
    .replace(/=/g, "")
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
}

function base64UrlDecode(str: string): string {
  str = str.replace(/-/g, "+").replace(/_/g, "/")
  while (str.length % 4) str += "="
  return atob(str)
}

async function hmacSha256(secret: string, data: string): Promise<string> {
  const enc = new TextEncoder()
  const key = await crypto.subtle.importKey("raw", enc.encode(secret), { name: "HMAC", hash: "SHA-256" }, false, ["sign"])
  const sig = await crypto.subtle.sign({ name: "HMAC", hash: "SHA-256" }, key, enc.encode(data))
  const bytes = Array.from(new Uint8Array(sig))
    .map((b) => String.fromCharCode(b))
    .join("")
  return base64UrlEncode(bytes)
}

export async function verifyToken(token: string, secret: string): Promise<Result<AuthToken>> {
  try {
    const parts = token.split(".")
    if (parts.length !== 3) {
      return { ok: false, error: new Error("Invalid token format") }
    }

    const [header, payload, signature] = parts
    const expectedSig = await hmacSha256(secret, `${header}.${payload}`)

    if (signature !== expectedSig) {
      return { ok: false, error: new Error("Invalid signature") }
    }

    const decoded = JSON.parse(base64UrlDecode(payload!)) as AuthToken & { type?: string }

    if (decoded.exp && decoded.exp * 1000 < Date.now()) {
      return { ok: false, error: new Error("Token expired") }
    }

    return {
      ok: true,
      value: {
        sub: decoded.sub,
        email: decoded.email,
        roles: decoded.roles ?? [],
        iat: decoded.iat,
        exp: decoded.exp,
        jti: decoded.jti,
      },
    }
  } catch (err) {
    return { ok: false, error: err instanceof Error ? err : new Error("Token verification failed") }
  }
}

export async function createToken(
  payload: Partial<AuthToken>,
  secret: string,
  expiresIn: string = "7d",
): Promise<string> {
  const now = Math.floor(Date.now() / 1000)
  const expMap: Record<string, number> = {
    "1h": 3600,
    "6h": 21600,
    "12h": 43200,
    "1d": 86400,
    "7d": 604800,
    "30d": 2592000,
  }
  const expSecs = expMap[expiresIn] ?? 604800

  const header = base64UrlEncode(JSON.stringify({ alg: "HS256", typ: "JWT" }))

  const body: Record<string, unknown> = {
    sub: payload.sub ?? "",
    email: payload.email ?? "",
    roles: payload.roles ?? [],
    iat: now,
    exp: now + expSecs,
    jti: payload.jti ?? crypto.randomUUID(),
  }

  const payloadEncoded = base64UrlEncode(JSON.stringify(body))
  const signature = await hmacSha256(secret, `${header}.${payloadEncoded}`)

  return `${header}.${payloadEncoded}.${signature}`
}

export async function hashPassword(password: string): Promise<string> {
  const enc = new TextEncoder()
  const salt = crypto.getRandomValues(new Uint8Array(16))
  const saltStr = Array.from(salt)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("")

  const key = await crypto.subtle.importKey(
    "raw",
    enc.encode(password + saltStr),
    { name: "PBKDF2" },
    false,
    ["deriveBits"],
  )

  const derived = await crypto.subtle.deriveBits(
    {
      name: "PBKDF2",
      salt: enc.encode(saltStr),
      iterations: 100000,
      hash: "SHA-256",
    },
    key,
    256,
  )

  const hash = Array.from(new Uint8Array(derived))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("")

  return `${saltStr}:${hash}`
}

export async function verifyPassword(password: string, stored: string): Promise<boolean> {
  const [salt, hash] = stored.split(":")
  if (!salt || !hash) return false

  const enc = new TextEncoder()
  const key = await crypto.subtle.importKey(
    "raw",
    enc.encode(password + salt),
    { name: "PBKDF2" },
    false,
    ["deriveBits"],
  )

  const derived = await crypto.subtle.deriveBits(
    {
      name: "PBKDF2",
      salt: enc.encode(salt),
      iterations: 100000,
      hash: "SHA-256",
    },
    key,
    256,
  )

  const computedHash = Array.from(new Uint8Array(derived))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("")

  return computedHash === hash
}

export interface AuthSession {
  user: { id: string; email: string; username: string }
  token: string
  expiresAt: number
}

export function parseAuthHeader(header: string): Result<string> {
  if (!header) {
    return { ok: false, error: new Error("No authorization header") }
  }

  const parts = header.split(" ")
  if (parts.length !== 2 || parts[0] !== "Bearer") {
    return { ok: false, error: new Error("Invalid authorization header format") }
  }

  return { ok: true, value: parts[1]! }
}
