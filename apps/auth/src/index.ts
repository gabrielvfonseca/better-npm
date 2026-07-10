import { Hono } from "hono"
import { HTTPException } from "hono/http-exception"
import { createLogger } from "@dash/logger"
import { loadConfig } from "@dash/config"
import { signToken, verifyToken, hashPassword, comparePassword, generateRefreshToken } from "@dash/auth"
import { validate } from "@dash/validation"
import type { StatusCode } from "hono/utils/http-status"

const logger = createLogger("auth")
const config = loadConfig()

const app = new Hono()

app.use("*", async (c, next) => {
  const start = performance.now()
  await next()
  const ms = (performance.now() - start).toFixed(2)
  logger.info(`${c.res.status} ${c.req.method} ${c.req.path} ${ms}ms`)
})

app.post("/auth/login", async (c) => {
  try {
    const body = await c.req.json()
    const { email, password } = validate("login", body)

    const user = await findUserByEmail(email)
    if (!user) {
      throw new HTTPException(401, { message: "Invalid email or password" })
    }

    const valid = await comparePassword(password, user.passwordHash)
    if (!valid) {
      throw new HTTPException(401, { message: "Invalid email or password" })
    }

    const accessToken = await signToken({ sub: user.id, email: user.email, role: user.role })
    const refreshToken = await generateRefreshToken({ sub: user.id })

    logger.info("User logged in", { userId: user.id })
    return c.json({ accessToken, refreshToken, expiresIn: 3600, tokenType: "Bearer" })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Login failed", { error: String(err) })
    throw new HTTPException(500, { message: "Authentication failed" })
  }
})

app.post("/auth/register", async (c) => {
  try {
    const body = await c.req.json()
    const { email, password, name } = validate("register", body)

    const existing = await findUserByEmail(email)
    if (existing) {
      throw new HTTPException(409, { message: "Email already registered" })
    }

    const passwordHash = await hashPassword(password)
    const user = await createUser({ email, name, passwordHash })

    const accessToken = await signToken({ sub: user.id, email: user.email, role: user.role })
    const refreshToken = await generateRefreshToken({ sub: user.id })

    logger.info("User registered", { userId: user.id })
    return c.json({ user: { id: user.id, email: user.email, name: user.name }, accessToken, refreshToken }, 201)
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Registration failed", { error: String(err) })
    throw new HTTPException(500, { message: "Registration failed" })
  }
})

app.post("/auth/refresh", async (c) => {
  try {
    const body = await c.req.json()
    const { refreshToken } = validate("refresh", body)

    const payload = await verifyToken(refreshToken)
    if (!payload || !payload.sub) {
      throw new HTTPException(401, { message: "Invalid or expired refresh token" })
    }

    const user = await findUserById(payload.sub)
    if (!user) {
      throw new HTTPException(401, { message: "User not found" })
    }

    const accessToken = await signToken({ sub: user.id, email: user.email, role: user.role })
    const newRefreshToken = await generateRefreshToken({ sub: user.id })

    return c.json({ accessToken, refreshToken: newRefreshToken, expiresIn: 3600, tokenType: "Bearer" })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Token refresh failed", { error: String(err) })
    throw new HTTPException(500, { message: "Token refresh failed" })
  }
})

app.post("/auth/logout", async (c) => {
  try {
    const body = await c.req.json().catch(() => ({}))
    const token = c.req.header("Authorization")?.replace("Bearer ", "")
    logger.info("User logged out", { token: token ? `${token.slice(0, 10)}...` : undefined })
    return c.json({ message: "Logged out successfully" })
  } catch (err) {
    logger.error("Logout failed", { error: String(err) })
    throw new HTTPException(500, { message: "Logout failed" })
  }
})

app.get("/auth/verify", async (c) => {
  try {
    const authHeader = c.req.header("Authorization")
    if (!authHeader?.startsWith("Bearer ")) {
      throw new HTTPException(401, { message: "Missing or invalid authorization header" })
    }

    const token = authHeader.slice(7)
    const payload = await verifyToken(token)
    if (!payload) {
      throw new HTTPException(401, { message: "Invalid or expired token" })
    }

    return c.json({ valid: true, payload })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Token verification failed", { error: String(err) })
    throw new HTTPException(500, { message: "Token verification failed" })
  }
})

app.get("/auth/session", async (c) => {
  try {
    const authHeader = c.req.header("Authorization")
    if (!authHeader?.startsWith("Bearer ")) {
      throw new HTTPException(401, { message: "Missing or invalid authorization header" })
    }

    const token = authHeader.slice(7)
    const payload = await verifyToken(token)
    if (!payload || !payload.sub) {
      throw new HTTPException(401, { message: "Invalid or expired token" })
    }

    const user = await findUserById(payload.sub)
    if (!user) {
      throw new HTTPException(404, { message: "User not found" })
    }

    return c.json({
      session: {
        user: { id: user.id, email: user.email, name: user.name, role: user.role },
        createdAt: payload.iat ? new Date(payload.iat * 1000).toISOString() : undefined,
        expiresAt: payload.exp ? new Date(payload.exp * 1000).toISOString() : undefined,
      },
    })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Session retrieval failed", { error: String(err) })
    throw new HTTPException(500, { message: "Session retrieval failed" })
  }
})

app.onError((err, c) => {
  if (err instanceof HTTPException) {
    return c.json({ error: err.message, status: err.status }, err.status as StatusCode)
  }
  logger.error("Unhandled error", { error: err.message })
  return c.json({ error: "Internal server error", status: 500 }, 500)
})

app.notFound((c) => c.json({ error: "Not found", status: 404 }, 404))

async function findUserByEmail(email: string) {
  return null
}

async function findUserById(id: string) {
  return null
}

async function createUser(data: { email: string; name: string; passwordHash: string }) {
  return { id: crypto.randomUUID(), email: data.email, name: data.name, role: "user", passwordHash: data.passwordHash }
}

export { app }
export default app
