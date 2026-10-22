import { BnpmAgentClient } from "./client.js"
import type { AgentClient, AgentClientConfig } from "./types.js"

export function createAgentClient(config: AgentClientConfig): AgentClient {
  return new BnpmAgentClient(config)
}
