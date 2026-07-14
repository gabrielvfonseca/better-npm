import { createToken, hashPassword, verifyPassword, verifyToken } from "@dash/auth"
import { loadConfig } from "@dash/config"
import { createLogger } from "@dash/logger"
import { createUserSchema, loginSchema } from "@dash/schemas"
import { validate } from "@dash/validation"
import { Hono } from "hono"
import { HTTPException } from "hono/http-exception"

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
    const loginResult = validate(loginSchema, body)
    if (!loginResult.ok) {
      throw new HTTPException(400, { message: "Invalid login data" })
    }
    const { email, password } = loginResult.value

    const user = await findUserByEmail(email)
    if (!user) {
      throw new HTTPException(401, { message: "Invalid email or password" })
    }

    const valid = await verifyPassword(password, user.passwordHash)
    if (!valid) {
      throw new HTTPException(401, { message: "Invalid email or password" })
    }

    const accessToken = await createToken(
      { sub: user.id, email: user.email, roles: [user.role] },
      config.auth.secret,
      config.auth.tokenExpiry,
    )

    logger.info("User logged in", { userId: user.id })
    return c.json({ accessToken, expiresIn: 3600, tokenType: "Bearer" })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Login failed", { error: String(err) })
    throw new HTTPException(500, { message: "Authentication failed" })
  }
})

app.post("/auth/register", async (c) => {
  try {
    const body = await c.req.json()
    const registerResult = validate(createUserSchema, body)
    if (!registerResult.ok) {
      throw new HTTPException(400, { message: "Invalid registration data" })
    }
    const { email, password } = registerResult.value

    const existing = await findUserByEmail(email)
    if (existing) {
      throw new HTTPException(409, { message: "Email already registered" })
    }

    const passwordHash = await hashPassword(password)
    const user = await createUser({ email, name: "", passwordHash })

    const accessToken = await createToken(
      { sub: user.id, email: user.email, roles: [user.role] },
      config.auth.secret,
      config.auth.tokenExpiry,
    )

    logger.info("User registered", { userId: user.id })
    return c.json({ user: { id: user.id, email: user.email, name: user.name }, accessToken }, 201)
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Registration failed", { error: String(err) })
    throw new HTTPException(500, { message: "Registration failed" })
  }
})

app.post("/auth/logout", async (c) => {
  try {
    c.req.header("Authorization")?.replace("Bearer ", "")
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
    const payload = await verifyToken(token, config.auth.secret)
    if (!payload.ok) {
      throw new HTTPException(401, { message: "Invalid or expired token" })
    }

    return c.json({ valid: true, payload: payload.value })
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
    const payload = await verifyToken(token, config.auth.secret)
    if (!(payload.ok && payload.value?.sub)) {
      throw new HTTPException(401, { message: "Invalid or expired token" })
    }

    const user = await findUserById(payload.value.sub)
    if (!user) {
      throw new HTTPException(404, { message: "User not found" })
    }

    return c.json({
      session: {
        user: { id: user.id, email: user.email, name: user.name, role: user.role },
        createdAt: payload.value.iat ? new Date(payload.value.iat * 1000).toISOString() : undefined,
        expiresAt: payload.value.exp ? new Date(payload.value.exp * 1000).toISOString() : undefined,
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
    return c.json({ error: err.message, status: err.status }, err.status)
  }
  logger.error("Unhandled error", { error: err.message })
  return c.json({ error: "Internal server error", status: 500 }, 500)
})

app.notFound((c) => c.json({ error: "Not found", status: 404 }, 404))

async function findUserByEmail(_email: string) {
  return null as {
    id: string
    email: string
    name: string
    role: string
    passwordHash: string
  } | null
}

async function findUserById(_id: string) {
  return null as {
    id: string
    email: string
    name: string
    role: string
    passwordHash: string
  } | null
}

async function createUser(data: { email: string; name: string; passwordHash: string }) {
  return {
    id: crypto.randomUUID(),
    email: data.email,
    name: data.name,
    role: "user",
    passwordHash: data.passwordHash,
  }
}

export { app }
export default app
