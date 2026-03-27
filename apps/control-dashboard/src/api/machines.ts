// Stub API module for Machine management — SMET-I-0039

export interface Machine {
  id: string
  name: string
  platform: string
  status: 'online' | 'stale' | 'offline' | 'pending'
  trustTier: 'trusted' | 'restricted' | 'pending'
  lastHeartbeat: string
  repos: string[]
}

export function listMachines(): Promise<Machine[]> {
  throw new Error('Not implemented — see SMET-I-0039')
}

export function getMachine(_id: string): Promise<Machine> {
  throw new Error('Not implemented — see SMET-I-0039')
}

export function approveMachine(_id: string): Promise<void> {
  throw new Error('Not implemented — see SMET-I-0039')
}

export function revokeMachine(_id: string): Promise<void> {
  throw new Error('Not implemented — see SMET-I-0039')
}
